# Etapa 2 — preparação para redes diferentes

## Estado e objetivo

A Etapa 1 foi aceita em dois Windows na mesma rede Wi-Fi. Nesta etapa, os dois
clientes usarão uma sinalização acessível pela internet e poderão usar TURN
próprio quando o caminho direto não funcionar.

A máquina de hospedagem está indisponível para inspeção no momento. Por isso,
implementamos primeiro a configuração e os diagnósticos locais. Ainda não há
endereço público confirmado nem serviço implantado ou aceitação entre redes.

## Configuração local do cliente

Sem `CHAMADA_CONFIG`, o aplicativo mantém o comportamento LAN da Etapa 1.
Quando definida, a variável indica um arquivo JSON lido por Rust a cada entrada
na sala, antes de abrir câmera e microfone. Configuração inválida encerra a
tentativa com mensagem na tela; não há retorno silencioso para LAN.

Na raiz, prepare uma cópia local, que fica fora do Git:

```powershell
New-Item -ItemType Directory -Force .local
Copy-Item config/cliente.exemplo.json .local/cliente.json
notepad .local/cliente.json
$env:CHAMADA_CONFIG = (Resolve-Path .local/cliente.json).Path
cd app
flutter run -d windows
```

Substitua os exemplos pelos valores reais antes de usar. A variável pertence ao
processo; o aplicativo precisa ser iniciado pelo mesmo terminal para herdá-la.
Um executável já aberto não recebe a alteração. Para retornar à LAN, feche o
aplicativo e remova a variável com `Remove-Item Env:CHAMADA_CONFIG` antes de abrir.
Não é preciso regenerar os bindings: a API Flutter–Rust não mudou.

| Campo | Uso |
| --- | --- |
| `token_sinalizacao` | Token compartilhado de acesso à sinalização; use WSS fora do loopback. |
| `servidores_ice` | Lista de servidores STUN/TURN; `urls`, `usuario` e `senha`. |
| `somente_retransmissao` | `true` exige TURN, para ensaiar retransmissão sem caminho direto. |

O JSON não é um arquivo `.env` e não é carregado automaticamente. Senhas ficam
apenas na cópia local; não coloque credenciais na URL nem nas mensagens de commit.
O token da sinalização e a senha TURN são credenciais distintas, sem contas ou
serviço de identidade adicional nesta POC. Campos desconhecidos são rejeitados
para evitar que um erro de digitação desative uma opção.

Esta integração aceita STUN e TURN por UDP (`stun:HOST:3478` e
`turn:HOST:3478?transport=udp`). TURN por TCP/TLS não faz parte desta entrega;
uma rede que bloqueie UDP poderá impedir a chamada. WSS protege a sinalização,
usando certificados confiáveis pelo Windows, sem desativar a validação TLS.

## Identificar o caminho da mídia

Após conectar, Rust consulta o par ICE realmente selecionado e envia um evento
`caminho_ice`. Flutter apenas apresenta o texto abaixo do estado da chamada.
O diagnóstico mostra os tipos dos candidatos, sem endereços ou credenciais:

- `direto`: nenhum dos dois candidatos selecionados é `relay`.
- `retransmissão TURN`: pelo menos um candidato selecionado é `relay`.

A consulta ocorre a cada segundo e atualiza a tela quando os tipos mudam. Ter
TURN no JSON ou reunir um candidato `relay` não comprova que ele transportou
mídia. Para validar TURN, configure `somente_retransmissao: true` nos dois
clientes, confirme o diagnóstico de retransmissão e observe áudio/vídeo nos
dois sentidos. O teste sintético local também exige receber o diagnóstico ICE.

## Sinalização com acesso controlado

`SINALIZACAO_TOKEN` ativa um token compartilhado, conferido na mensagem `entrar`
antes de acessar a sala. Token ausente ou incorreto recebe `Acesso não autorizado.`
e a conexão fecha. Token vazio na configuração do servidor impede a inicialização;
não desativa a proteção. O token não é encaminhado ao outro participante ou impresso.

Com token definido, o endereço padrão passa a ser `127.0.0.1:8080`, para um proxy
WSS na mesma máquina. Sem a variável, permanece `0.0.0.0:8080` para o teste LAN.
O primeiro argumento ainda permite escolher explicitamente a interface/porta.
Para experimentar o controle de acesso em loopback, em um terminal PowerShell:

```powershell
$env:SINALIZACAO_TOKEN = 'TOKEN_APENAS_PARA_TESTE_LOCAL'
cargo run -p sinalizacao -- 127.0.0.1:8081
```

No JSON dos clientes use o mesmo `token_sinalizacao`, `servidores_ice: []` e
`somente_retransmissao: false`. Abra pelo terminal com `CHAMADA_CONFIG` definido
e informe `ws://127.0.0.1:8081` na tela. A exceção sem TLS é restrita ao loopback;
fora dele, um cliente com token exige `wss://`. Para usar dois dispositivos
físicos com token, prepare WSS. Ao voltar ao servidor LAN sem autenticação,
remova também `SINALIZACAO_TOKEN` do terminal do servidor.

## Implantação quando a máquina estiver disponível

Primeiro registre sistema, acesso administrativo, interfaces, IP público ou NAT,
domínio e regras permitidas. CGNAT pode impedir o recebimento de conexões; isso
precisa ser conferido na infraestrutura disponível antes de prometer acesso externo.

A proposta mínima é sinalização Rust atrás de um proxy WSS e coturn separado.
Caddy é uma opção para o proxy, com suporte a WebSocket e certificado público.
Os exemplos abaixo são preparação, ainda não executada na hospedagem. Instale
versões compatíveis com o sistema confirmado e registre as versões reais em
`04-validacao.md`. Em Windows, o modo de hospedar coturn ainda precisa ser escolhido;
não presumimos uma instalação nativa disponível.

Compile o servidor na plataforma da hospedagem com
`cargo build -p sinalizacao --release`. Defina um token aleatório local de pelo menos 32 caracteres em
`SINALIZACAO_TOKEN` e execute o binário em `127.0.0.1:8080`. O token não deve
aparecer na URL ou em arquivos versionados. Mantenha o backend acessível apenas
ao proxy da própria máquina.

Se o domínio apontar para essa máquina e ela puder receber TCP 80/443, um
`Caddyfile` local pode conter, substituindo o domínio pelo real:

```caddyfile
chamada.exemplo.org {
    reverse_proxy 127.0.0.1:8080
}
```

Execute `caddy validate --config CAMINHO_DO_CADDYFILE --adapter caddyfile` e
`caddy run --config CAMINHO_DO_CADDYFILE --adapter caddyfile`. TCP 80 pode ser
necessário para emissão/renovação automática do certificado. Os clientes usarão
`wss://chamada.exemplo.org`; não exponha o backend TCP 8080. Se já houver proxy e
certificado na máquina, adapte o serviço existente.

Para coturn em um sistema compatível, prepare um `turnserver.conf` local. Este
exemplo usa credencial estática exclusiva da POC para evitar um serviço emissor
adicional. Substitua usuário/senha e restrinja acesso ao arquivo na hospedagem:

```ini
listening-port=3478
fingerprint
lt-cred-mech
realm=chamada-poc
user=USUARIO_DA_POC:SENHA_ALEATORIA_EXCLUSIVA
min-port=49160
max-port=49200
user-quota=8
total-quota=16
no-tcp
no-tls
no-dtls
no-tcp-relay
no-multicast-peers
no-cli
log-file=stdout
```

Com várias interfaces, defina `listening-ip` e `relay-ip` com o IP correto da
máquina. Atrás de NAT, configure `external-ip=IP_PUBLICO/IP_PRIVADO` e mapeie
UDP 3478 e UDP 49160–49200 preservando as portas. Libere essa mesma faixa no
firewall da hospedagem. O intervalo pequeno é para esta POC de duas pessoas,
sem avaliação de capacidade de produção. Não habilite `no-auth` ou peers
loopback. Restrinja acesso às redes privadas/serviços internos conforme a
topologia da hospedagem, sem liberar um relay anônimo.

Inicie com `turnserver -c CAMINHO_DO_ARQUIVO` e confira os logs. Nos clientes,
configure STUN/TURN com o host público e as mesmas credenciais TURN. Este exemplo
usa UDP somente; WSS da sinalização não torna o transporte TURN TCP/TLS.
TURN oferece o caminho de retransmissão quando não houver conexão direta.

## Aceitação da Etapa 2

1. Abra dois Windows em redes diferentes, com a mesma sala e WSS.
2. Com `somente_retransmissao: false`, observe áudio e vídeo nos dois sentidos
   por cinco minutos; registre o caminho ICE selecionado.
3. Encerre e entre novamente. Confira liberação dos dispositivos e nova chamada.
4. Feche os aplicativos, configure `somente_retransmissao: true` nos dois JSONs
   e repita. Exija `retransmissão TURN` nos dois clientes e mídia nos dois sentidos.
5. Registre duração, redes, versões, problemas, encerramento e repetição em
   `04-validacao.md`. Não copie senhas, tokens ou SDP com dados privados para o Git.

Um teste que só conecta ao servidor não comprova mídia. Um caminho direto não
comprova TURN. Até executar esse roteiro, a Etapa 2 permanece em implementação.

## Referências técnicas

- [Configuração ICE do webrtc-rs](https://docs.rs/webrtc/0.21.1/webrtc/peer_connection/index.html)
- [Configuração oficial do coturn](https://github.com/coturn/coturn/blob/master/examples/etc/turnserver.conf)
- [Proxy WebSocket com Caddy](https://caddyserver.com/docs/caddyfile/directives/reverse_proxy)

As assinaturas foram conferidas no código local das versões fixadas de `webrtc`
e `rtc` 0.21.1. Resultados efetivamente executados ficam em `04-validacao.md`.
