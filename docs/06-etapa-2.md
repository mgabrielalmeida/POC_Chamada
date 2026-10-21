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

## Referências técnicas

- [Configuração ICE do webrtc-rs](https://docs.rs/webrtc/0.21.1/webrtc/peer_connection/index.html)
- [Configuração oficial do coturn](https://github.com/coturn/coturn/blob/master/examples/etc/turnserver.conf)

As assinaturas foram conferidas no código local das versões fixadas de `webrtc`
e `rtc` 0.21.1. Resultados efetivamente executados ficam em `04-validacao.md`.
