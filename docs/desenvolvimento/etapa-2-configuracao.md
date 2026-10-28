# Etapa 2 — configuração e contratos para redes diferentes

A implantação escolhida usa um terceiro Windows doméstico, roteador acessível
e entrada pela internet, sem VPN. Sinalização Rust, Caddy e coturn nativo ficarão
nesse computador. O [roteiro detalhado](roteiro-entre-redes.md) explica
a preparação IPv6 do Windows, CGNAT IPv4, DNS AAAA, compilação e permissões
de entrada nos firewalls Windows/roteador. Não usa Port Forwarding IPv4.

A configuração abaixo já está implementada no cliente. O
[diagnóstico de 23/10](diagnostico-primeira-chamada.md) registra a implantação
investigada e ensaios posteriores à auditoria; o aceite físico completo entre
A/B continua pendente. Instruções não equivalem a evidências de execução atual.

[Desenvolvimento](README.md) · [Estudo da infraestrutura](../estudos/infraestrutura/README.md)

## Configuração local do cliente

Sem `CHAMADA_CONFIG`, o aplicativo mantém o comportamento LAN da Etapa 1.
Quando definida, a variável indica um JSON lido por Rust a cada entrada na sala,
antes de abrir câmera e microfone. Uma configuração inválida causa erro;
não há retorno silencioso ao modo LAN.

Na raiz do checkout, criar uma cópia local, fora do Git:

```powershell
New-Item -ItemType Directory -Force .local
Copy-Item config/cliente.exemplo.json .local/cliente.json
notepad .local/cliente.json
$env:CHAMADA_CONFIG = (Resolve-Path .local/cliente.json).Path
Push-Location app
flutter run -d windows
Pop-Location
```

Se a cópia já existir com dados próprios, editá-la em vez de sobrescrevê-la.
Preencher os valores reais antes de executar.

A variável pertence ao processo. Abrir o aplicativo pelo mesmo terminal permite
herdá-la; uma instância já aberta não recebe a alteração. Para voltar à LAN,
fechar e remover `CHAMADA_CONFIG` do terminal antes de abrir novamente.

| Campo | Uso |
| --- | --- |
| `token_sinalizacao` | Chave compartilhada de acesso à sinalização |
| `servidores_ice` | Lista com URLs e credenciais STUN/TURN |
| `somente_retransmissao` | Quando `true`, aplica política ICE Relay para testar TURN |

O JSON não é um arquivo `.env` e não é encontrado automaticamente.
Campos desconhecidos são rejeitados. O token da sinalização e a senha TURN
são credenciais diferentes; não há serviço de contas nesta POC.
Não é necessário regenerar bindings para alterar essa configuração.

O [modelo JSON](../../config/cliente.exemplo.json) deve receber nomes alcançáveis
pela internet. O domínio público deve ter AAAA para o IPv6 global não temporário do
Windows S. Os dois participantes precisam de conectividade IPv6 funcional.

## Transportes atualmente aceitos

A integração aceita STUN e TURN por UDP:

- `stun:HOST_PUBLICO:3478`
- `turn:HOST_PUBLICO:3478?transport=udp`

TURN por TCP/TLS não foi implementado e validado nesta entrega. WSS protege
a sinalização e não transforma a mídia em tráfego TURN TCP/TLS.
Uma rede que bloqueie UDP pode permitir entrada na sala e impedir a chamada.

WSS usa as raízes confiáveis do Windows, sem desativar validação TLS.
A proposta atual usa certificado público para um domínio; a autoridade interna
do Caddy e a VPN do roteiro anterior não fazem parte desse desenho.

## Diagnóstico do caminho ICE

Depois de conectar, Rust consulta o par ICE selecionado aproximadamente a cada
segundo. Quando o par muda, envia `caminho_ice`; Flutter apresenta o texto
separadamente do estado da chamada. O texto inclui tipo, protocolo, família,
endereço e porta dos candidatos local/remoto. Não inclui credenciais.

- `direto`: nenhum candidato selecionado é `relay`.
- `retransmissão TURN`: pelo menos um candidato selecionado é `relay`.

Configurar TURN ou reunir um candidato relay não comprova seu uso.
Para validá-lo, exigir retransmissão forçada nos dois clientes, indicação
compatível e áudio/vídeo nos dois sentidos.

## Sinalização com token

`SINALIZACAO_TOKEN` ativa uma chave compartilhada conferida na mensagem
`entrar`, antes de acessar a sala. Token ausente ou incorreto produz
`Acesso não autorizado.` e fecha a conexão. Token configurado como vazio impede
o início do servidor. O valor não é encaminhado ao outro participante.

Com token, a escuta padrão é `127.0.0.1:8080`, apropriada ao Caddy na mesma
máquina. Sem a variável, permanece `0.0.0.0:8080` para LAN.
O primeiro argumento ainda permite escolher explicitamente interface e porta.

Para verificar autorização em loopback, em PowerShell:

```powershell
$env:SINALIZACAO_TOKEN = 'TOKEN_APENAS_PARA_TESTE_LOCAL'
cargo run -p sinalizacao -- 127.0.0.1:8081
```

Nos clientes locais desse ensaio, usar o mesmo token, `servidores_ice: []`,
`somente_retransmissao: false` e `ws://127.0.0.1:8081`.
Essa exceção sem TLS é restrita ao loopback. Fora dele, o cliente com token
exige `wss://`. Ao retornar ao servidor LAN sem autenticação, remover também
`SINALIZACAO_TOKEN` do ambiente correspondente.

## Contrato da implantação no terceiro Windows

| Componente | Configuração de referência |
| --- | --- |
| Rust | `sinalizacao.exe 127.0.0.1:8080`, com token no ambiente |
| Caddy | Domínio AAAA com certificado público; proxy para o backend local |
| coturn | Nativo Windows 4.18.0; listening-ip/relay-ip no IPv6 global de S |
| Alocações TURN | `allocation-default-address-family=ipv6`; sem external-ip/NAT |
| Firewalls Windows e roteador | Autorizar IPv6 TCP 80/443 e UDP 3478/49160–49200 para S |
| Clientes | Conectividade IPv6, mesmo domínio público, sala e credenciais |

O CGNAT IPv4 foi identificado pelo responsável. A implantação passou a usar
IPv6 global, sem tradução de endereço. Endereço global não comprova permissão
de entrada: testar os serviços a partir das duas redes externas. TCP 8080
permanece local. Redes exclusivamente IPv4 não alcançam esta hospedagem.

A biblioteca `rtc-turn 0.21.1` não solicita família na mensagem Allocate;
por isso o coturn dedicado deve ter o padrão IPv6 explicitamente configurado.
O cliente abre sockets das duas famílias; a coleta STUN/TURN resolve o hostname
pelo runtime Tokio e escolhe um endereço da família do socket. Host, srflx e
relay IPv6 usam os mesmos SDP/ICE/DTLS/SRTP, sem conversão no Flutter.

TURN IPv6 literal é rejeitado com orientação para usar hostname AAAA: a versão
fixada de webrtc remonta o endpoint sem os colchetes. STUN literal com colchetes
e WSS IPv6 são aceitos; certificado WSS continua sendo validado. Os detalhes
da investigação e os limites estão na [auditoria](auditoria-ipv6.md).

A compilação Windows de referência está descrita no
[workflow oficial do coturn 4.18.0](https://github.com/coturn/coturn/blob/4.18.0/.github/workflows/msvc.yml).
Não habilitar CLI ou DTLS; não reutilizar `no-cli`/`no-dtls` de exemplos antigos. Consulte as
[notas de versão](https://github.com/coturn/coturn/releases/tag/4.18.0).
O passo a passo e a configuração completa estão no roteiro detalhado.

## Aceitação

1. Usar dois Windows participantes em redes diferentes, sem VPN, com o mesmo WSS e sala.
2. No modo normal, observar áudio/vídeo bidirecional por cinco minutos e registrar o par ICE.
3. Encerrar, conferir liberação dos dispositivos e repetir a chamada.
4. Repetir por cinco minutos com `somente_retransmissao: true` nos dois clientes,
   exigindo relay local/remoto IPv6 em ambos, contadores TURN e áudio/vídeo bidirecional.
5. Registrar rede, versões, duração, problemas e resultados em [validação](validacao.md).
6. Se a inicialização automática for preparada, verificar separadamente sua execução após reboot.

Um teste de sinalização ou de porta TCP não comprova mídia UDP.
A etapa continua em implementação até os critérios de chamada serem executados
e aceitos. Não versionar credenciais nem SDP com dados privados.
