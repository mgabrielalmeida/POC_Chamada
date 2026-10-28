# Auditoria IPv6 — implementação e implantação

[Desenvolvimento](README.md) · [Estudo das dificuldades](../estudos/webrtc/dificuldades-e-decisoes.md)

Este relatório preserva o estado da auditoria inicial de 23/10/2026. Os avanços
da investigação posterior, incluindo coturn real e ensaios sintéticos no S,
estão no [diagnóstico da primeira chamada](diagnostico-primeira-chamada.md).
As pendências abaixo são as daquela rodada, não uma inspeção atual dos serviços.

Data: 23/10/2026. Escopo: PoC Windows 1:1, Flutter + Rust + webrtc-rs,
sinalização Rust, Caddy e coturn 4.18.0 nativos. Sem VPN. Não houve acesso ao
Windows S, ao FGA2110 ou aos participantes externos. A entrada IPv4 está sob
CGNAT conforme investigação fornecida pelo responsável. Os IPs daquela
observação não foram incorporados ao código nem aos exemplos.

## Resultado da inspeção antes das mudanças

| Caminho auditado | Evidência concreta | Decisão |
| --- | --- | --- |
| Interface e FRB | `main.dart` envia a URL como String; `api.rs` encaminha sem separar IP por ponto/dois-pontos | Preservados, inclusive bindings |
| Configuração de WSS | `rede.rs` usa `http::Uri`; `connect_async` usa resolução do Tokio e rustls com raízes do sistema | Preservados; acrescentados casos IPv6 e TLS/token |
| Backend | `TcpListener::bind` aceita socket IPv6; salas trabalham com JSON, não com família IP | Preservado em 127.0.0.1:8080 atrás de Caddy; teste de sala/token IPv6 |
| Bind WebRTC | `chamada.rs` passava só `0.0.0.0:0` | Corrigido para wildcards IPv4 e IPv6 |
| DNS ICE | `webrtc 0.21.1/src/runtime/tokio.rs::resolve_host` usa `tokio::net::lookup_host` | Já retorna IPv4/IPv6; não criar resolver próprio |
| Interfaces | `peer_connection/driver.rs::resolve_bind_addrs/expand_wildcard` expande cada wildcard na mesma família, excluindo loopback/link-local | Preservado; o novo wildcard disponibiliza interfaces IPv6 reais |
| STUN srflx | `transport/stun_gatherer.rs` resolve todos os endereços, escolhe família compatível e decodifica XOR-MAPPED-ADDRESS | Preservado; teste de STUN UDP IPv6 local com resposta controlada |
| TURN via hostname | `transport/turn_relayer.rs::gather` resolve hostname e combina socket/servidor da mesma família | Compatível com AAAA quando há socket IPv6 |
| Família relay | `rtc-turn 0.21.1/src/client/mod.rs::allocate/handle_allocate_response` não inclui REQUESTED-ADDRESS-FAMILY | Configurar padrão IPv6 no coturn dedicado |
| TURN literal IPv6 | `turn_relayer.rs` reconstrói `format!("{}:{}", url.host, url.port)`; o parser remove os colchetes | Rejeitar antecipadamente com orientação AAAA; limitação upstream permanece |
| Candidatos/SDP | `rtc-ice` distingue Udp4/Udp6 e não combina famílias incompatíveis; candidata ICE usa IP como texto, SDP completo circula sem transformação | Preservados; ensaio local confere a família do par selecionado |
| Diagnóstico | Só mostrava host/srflx/relay | Acrescentar protocolo, IPv4/IPv6, IP e porta de ambos e detectar mudança do par |
| Mídia e helpers | FFmpeg usa loopback IPv4 apenas para RTP local; transporte entre PCs passa por ICE/DTLS/SRTP | Preservados; IPv4 local dos helpers não restringe a rede externa |
| coturn antigo | Endereços privados e external-ip pressupunham NAT IPv4 | Substituir por IPv6 global atribuído a S, sem external-ip |
| Caddy/DNS | Proxy local já correto; roteiro publicava A e exigia Port Forwarding IPv4 | Manter proxy e TLS público; substituir implantação por AAAA/permissões IPv6 |

A inspeção foi feita nos fontes **efetivamente instalados** no registro Cargo,
com as versões fixadas no Cargo.lock. Não se presumiu que um tipo IP genérico
ou a descrição da biblioteca demonstrasse suporte de ponta a ponta.

## Escolhas para manter o incremento pequeno

O JSON mantém seus campos; não há dependência nova, fork, vendor da biblioteca,
resolver próprio, protocolo novo nem regeneração da ponte. A configuração LAN
continua sem exigir arquivo ou serviços externos. O cliente agora também coleta
IPv6 quando disponível. O backend e a captura não precisavam mudar.

Em TURN, alcançar o servidor em IPv6 e pedir relay IPv6 são decisões diferentes.
O cliente fixado omite a segunda. O servidor dedicado usa explicitamente
`allocation-default-address-family=ipv6`, suportado no parser 4.18.0. Isso
altera o padrão IPv4 normativo para pedidos sem atributo; não é anunciado como
suporte completo à solicitação RFC 6156 pelo cliente. Não há promessa de relay
entre famílias ou de atendimento a participantes exclusivamente IPv4.

Para o erro de literal TURN, usar hostname AAAA satisfaz a topologia escolhida.
A validação específica evita aguardar coleta vazia/timeout sem explicação. STUN
com literal IPv6 e WSS com IPv6 entre colchetes continuam aceitos no parser.
Certificado WSS ainda precisa ser confiável e válido para o endpoint acessado.

## Configuração entregue

- [Cliente JSON](../../config/cliente.exemplo.json): domínio ilustrativo, token e
  credenciais a preencher em cópia privada.
- [coturn IPv6](../../config/turnserver-ipv6.exemplo.conf): escuta/relay globais,
  padrão IPv6, autenticação, quotas finitas e filtros de destinos não públicos.
- [Caddyfile](../../config/Caddyfile.exemplo): TLS público para domínio e backend local.
- [Roteiro integral](roteiro-entre-redes.md): instruções por máquina,
  AAAA, atualização de prefixo, build nativo TURN, firewalls e cinco cenários de aceite.

Não foram aplicadas regras no firewall real ou no roteador. Os menus e capacidades
do firmware FGA2110 da operadora permanecem por conferir. A documentação orienta
o efeito necessário e não inventa uma sequência de telas.

## Verificações reproduzíveis

Na raiz, com toolchain Windows:

```powershell
cargo fmt --all -- --check
cargo check --locked --workspace
cargo test --locked --workspace
cargo test --locked -p chamada --test chamada_sintetica -- --ignored --nocapture
cargo build --locked -p sinalizacao --release
```

`rust/tests/ipv6.rs` cria duas PeerConnections com bind exclusivo em `[::1]`,
negocia trilhas VP8/Opus, envia payloads RTP conhecidos nos dois sentidos,
confere conteúdo recebido após DTLS/SRTP, exige par host IPv6 selecionado,
fecha e repete. **Não codifica nem decodifica áudio/vídeo real.** O teste
sintético existente complementa isso com VP8/Opus gerados e decodificados,
FFmpeg e duas chamadas no caminho completo da aplicação.

O ensaio STUN local responde com XOR-MAPPED-ADDRESS IPv6 de documentação,
verificando que a coleta por hostname produz `srflx` IPv6 no SDP. Esse endereço
controlado não representa um mapeamento público observado. O teste de resolução
usa `localhost` no mesmo runtime do ICE; DNS AAAA público depende da implantação.
O teste de sinalização IPv6 exercita autenticação, papéis e encaminhamento de SDP
com sockets reais. Nenhum deles emite certificado público ou testa os firewalls WAN.

Os resultados realmente executados, incluindo falhas intermediárias e limites,
estão em [validação](validacao.md). A suíte comum ignora propositalmente
o teste FFmpeg e o ensaio coturn, cujos pré-requisitos são externos à suíte.

### Ensaio opcional com coturn local real

Para executar `rtp_bidirecional_por_turn_ipv6_real`, preparar um processo coturn
**isolado e restrito a loopback**, sem expor portas. Exemplo de arquivo privado:

```ini
listening-port=3479
listening-ip=::1
relay-ip=::1
allocation-default-address-family=ipv6
fingerprint
lt-cred-mech
realm=teste-local
user=teste:SENHA_LOCAL_DE_TESTE
min-port=49210
max-port=49230
user-quota=8
total-quota=16
no-tcp
no-tls
no-tcp-relay
no-multicast-peers
allow-loopback-peers
log-file=stdout
```

`allow-loopback-peers` é exclusivo deste processo local de teste e não deve
ser levado ao servidor público. Não reutilizar o arquivo público com suas
credenciais ou filtros neste teste. Iniciar coturn com `-c` apontando ao arquivo
local. Em outro terminal:

```powershell
$env:CHAMADA_TESTE_TURN_PORTA = '3479'
$env:CHAMADA_TESTE_TURN_USUARIO = 'teste'
$env:CHAMADA_TESTE_TURN_SENHA = Read-Host 'Senha do coturn local de teste'
cargo test --locked -p chamada --test ipv6 rtp_bidirecional_por_turn_ipv6_real -- --ignored --nocapture
Remove-Item Env:CHAMADA_TESTE_TURN_SENHA
```

Esse ensaio exige relay local/remoto IPv6 selecionado e recebimento do payload
RTP nas duas trilhas de ambos. Não inicia coturn nem substitui os cinco cenários
físicos. Encerrar o coturn local depois do teste.

## Pendências externas que impedem declarar a Etapa 2 aceita

1. Compilar/empacotar e executar coturn 4.18.0 nativo em S, registrar versão/DLLs.
2. Selecionar o IPv6 atual, publicar AAAA e obter certificado público no Caddy.
3. Aplicar permissões IPv6 específicas nos firewalls de S/FGA2110 e conferir o provedor.
4. A/B externos: WSS autenticado, ICE normal e TURN forçado com par IPv6 relay
   local/remoto, contadores crescentes e áudio/vídeo reais nos dois sentidos.
5. Medir cinco minutos em cada modo e repetir chamada sem reiniciar serviços.
6. Se configuradas, conferir tarefas de inicialização após reiniciar S.

## Arquivos tocados nesta tarefa

Alterações anteriores de documentação foram preservadas e adaptadas; nenhum commit foi criado.

| Arquivo | Alteração |
| --- | --- |
| [rust/src/chamada.rs](../../rust/src/chamada.rs) | Bind IPv4/IPv6 e descrição completa do par selecionado |
| [rust/src/rede.rs](../../rust/src/rede.rs) | Validação explicada do limite TURN literal e casos IPv6 |
| [rust/tests/ipv6.rs](../../rust/tests/ipv6.rs) | Resolução, STUN/srflx, RTP bidirecional IPv6 e ensaio TURN opcional |
| [servidor/tests/salas.rs](../../servidor/tests/salas.rs) | Mesma verificação de token/sala também em WebSocket IPv6 |
| [config/cliente.exemplo.json](../../config/cliente.exemplo.json) | Hostname ilustrativo para AAAA, sem mudar campos |
| [config/turnserver-ipv6.exemplo.conf](../../config/turnserver-ipv6.exemplo.conf) | Modelo IPv6 sem NAT, com autenticação/quotas/filtros |
| [config/Caddyfile.exemplo](../../config/Caddyfile.exemplo) | Modelo de domínio público e proxy loopback |
| [README.md](../../README.md) | Estado atual e entrada para roteiro/auditoria |
| [docs/estudos/arquitetura/README.md](../estudos/arquitetura/README.md) | Topologia IPv6 e separação dos serviços |
| [docs/desenvolvimento/etapas.md](etapas.md) | Pré-condições e aceite entre redes via IPv6 |
| [docs/estudos/README.md](../estudos/README.md) | Caminho dos sockets/diagnósticos e novos testes |
| [docs/desenvolvimento/validacao.md](validacao.md) | Novas evidências/limites sem reescrever históricos |
| [docs/desenvolvimento/etapa-2-configuracao.md](etapa-2-configuracao.md) | Contratos AAAA, família relay e limitações |
| [docs/desenvolvimento/roteiro-entre-redes.md](roteiro-entre-redes.md) | Revisão integral da operação em três Windows |
| [docs/desenvolvimento/auditoria-ipv6.md](auditoria-ipv6.md) | Este relatório de inspeção, decisões e entrega |

## Fontes primárias consultadas

- [Exemplo de configuração coturn 4.18.0](https://github.com/coturn/coturn/blob/4.18.0/examples/etc/turnserver.conf)
- [Parser/help coturn 4.18.0](https://github.com/coturn/coturn/blob/4.18.0/src/apps/relay/mainrelay.c)
- [Build MSVC upstream 4.18.0](https://github.com/coturn/coturn/blob/4.18.0/.github/workflows/msvc.yml)
- [Notas 4.18.0](https://github.com/coturn/coturn/releases/tag/4.18.0)
- [HTTPS automático Caddy](https://caddyserver.com/docs/automatic-https)
- [Escuta Caddy](https://caddyserver.com/docs/caddyfile/directives/bind)
- [ACME com IPv6](https://letsencrypt.org/docs/ipv6-support/)

As afirmações sobre webrtc/rtc-turn acima vêm dos fontes locais 0.21.1, não
de documentação de uma versão mais recente. Os registros históricos de LAN
e da proposta IPv4 anterior continuam preservados em [validação](validacao.md).
