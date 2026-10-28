# Ambiente e validação

[Desenvolvimento](README.md) · [Procedimentos de teste](testes.md)

Este é um histórico: cada seção descreve o estado da sua rodada, inclusive
pendências que podem ter avançado depois. O registro posterior à auditoria
IPv6 está em [diagnóstico da primeira chamada](diagnostico-primeira-chamada.md).
Reorganizar os documentos em 27/10/2026 não reexecutou essas verificações.

## Base inicial — 21/10/2026

Ferramentas observadas nesta máquina:

| Ferramenta | Versão |
| --- | --- |
| Flutter | 3.47.6, canal stable |
| Dart | 3.13.5 |
| rustc | 1.98.1 |
| Cargo | 1.98.1 |
| Git | 2.55.0.windows.3 |
| Visual Studio Build Tools | 2026, 18.8.0 |
| Windows SDK | 10.0.26100.0 |

`flutter doctor -v` reconheceu as ferramentas de compilação Windows.
`flutter devices` reconheceu o alvo `windows-x64`. O diagnóstico também apontou
pendências de Android e Chrome, alvos fora do escopo desta POC.

A execução inicial do Flutter não conseguiu escrever no cache do SDK dentro do
ambiente restrito. A verificação foi repetida com acesso ao SDK e concluída.

## Verificações da base

| Comando | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | Aprovado após instalar o componente `rustfmt`, ausente no ambiente. |
| `cargo check --workspace` | Biblioteca cliente e executável servidor verificados sem erros. |
| `cargo run -p sinalizacao` | Exibiu a mensagem de base ainda sem sinalização e encerrou normalmente. |
| `flutter pub get` em `app/` | Dependências resolvidas e lockfile atualizado. |
| `dart format --output=none --set-exit-if-changed lib` em `app/` | Aprovado após aplicar a formatação. |
| `flutter analyze` em `app/` | Nenhum problema encontrado. |
| `flutter build windows --debug` em `app/` | Executável Windows gerado com sucesso. |
| `git diff --cached --check` | Sem problemas de espaços em branco nos arquivos preparados. |

Executável gerado: `app/build/windows/x64/runner/Debug/poc_chamada.exe`.
Saídas de compilação ficam fora do Git. Os lockfiles Flutter e Rust são versionados.
A formatação Dart também precisou de acesso ao cache local da ferramenta.

## Limites da base inicial, antes da Etapa 1

- Flutter-Rust Bridge e webrtc-rs ainda não estão instalados ou integrados.
- Nenhuma captura, reprodução, conexão de sinalização ou chamada foi testada.
- A máquina que hospedará a segunda etapa ainda não foi inspecionada.
- Aceitação visual e chamada em dois dispositivos físicos permanecem pendentes.

## Etapa 1 — ponte

- Gerados bindings FRB 2.13.0 para Rust e Dart.
- `cargo build -p chamada`: DLL compilada.
- `dart run tool/verificar_ponte.dart`: chamada real retornou a mensagem esperada.
- O gerador precisou instalar `cargo-expand` antes da primeira geração.
- A análise apontou imports relativos no utilitário de verificação; foram
  corrigidos para imports `package:` antes da nova análise.
- Nova execução de `flutter analyze`: nenhum problema encontrado.

## Etapa 1 — sinalização

- `cargo check -p sinalizacao`: aprovado.
- `cargo test -p sinalizacao`: dois testes de integração aprovados com sockets
  WebSocket reais em loopback e portas escolhidas pelo sistema.
- Cobertos: papéis oferta/resposta, encaminhamento de SDP, sala lotada, nome
  inválido, mensagem malformada, aviso de saída e reutilização da sala.
- Estes testes não comprovam acesso pela LAN, firewall ou mídia.

## Etapa 1 — mídia e WebRTC

- Fixados FRB 2.13.0, `webrtc`/`rtc` 0.21.1 e FFmpeg 9.0.2 essentials.
- `scripts/preparar-midia.ps1`: SHA256 conferido e preparação reproduzida.
- `cargo check -p chamada`: integração com webrtc-rs compilada.
- `cargo test -p chamada --test chamada_sintetica -- --ignored --nocapture`:
  duas sessões reais em loopback trocaram mídia VP8/Opus, decodificaram vídeo e
  áudio PCM dos dois lados, encerraram e repetiram a chamada. O teste exige pelo
  menos cinco quadros e cinco blocos PCM remotos em cada sessão, em cada rodada.
- O teste inicial revelou um aviso de porta RTP/RTCP local e erros iniciais de
  decodificação. Foram reservados pares de portas, separadas as portas de saída
  do encoder e preservados os primeiros quadros de entrada. A repetição passou
  sem esses avisos. O envio filtra pacotes que não sejam VP8/Opus RTP.
- `cargo run -p chamada --example diagnosticar_midia`: câmera real retornou
  307.200 bytes RGBA (320×240×4) e microfone real retornou um pacote RTP Opus
  válido com 44 bytes de payload. Os helpers foram encerrados ao terminar.
- Dispositivos observados: ACER HD User Facing e Microphone Array (AMD Audio Device).
- `dart run tool/verificar_ponte.dart`: confirmou a chamada simples, o stream,
  a propagação de erro Rust e o encerramento sem abrir dispositivos.
- A verificação adicional encontrou um sink Rust mantendo o stream aberto no
  cancelamento Dart. Foi incluído `encerrar_observacao`, chamado antes de cancelar
  a assinatura. O utilitário passou e terminou após a correção.

## Etapa 1 — interface e pacote Windows

- Interface mínima com servidor, sala, criar/entrar, encerrar e vídeos local/remoto.
- `flutter analyze`: nenhum problema encontrado na versão final.
- `cargo fmt --all -- --check` e `git diff --check`: aprovados.
- `flutter build windows --debug`: aprovado na versão final. Executável em
  `app/build/windows/x64/runner/Debug/poc_chamada.exe`, acompanhado de `chamada.dll`,
  FFmpeg, FFplay e licença FFmpeg copiada integralmente.
- Um build intermediário não conseguiu sobrescrever a DLL porque o utilitário
  Dart de verificação ainda a mantinha carregada. Após corrigir o fechamento do
  stream e finalizar o utilitário, o build foi repetido com sucesso. Não execute
  esse utilitário ao mesmo tempo que compila a DLL no Windows.
- Não foi feita inspeção visual da janela nem aceitação humana de áudio/vídeo.

## Aceitação da Etapa 1 — 21/10/2026

O responsável informou: "A etapa 1 foi concluída e testada, está tudo funcionando
entre dois dispositivos windows em uma mesma rede wi-fi."

Com base nessa confirmação humana, a Etapa 1 está concluída. O relato comprova
o funcionamento no cenário LAN informado, além das evidências automatizadas
registradas acima. Não foram fornecidos duração medida, modelos do segundo
dispositivo ou detalhes do pacote utilizado; esses dados não foram presumidos.

## Limites de evidência preservados após a Etapa 1

- O responsável confirmou a conclusão da chamada na LAN; não há registro
  individual dos tempos e das observações de cada critério de aceitação.
- O teste sintético decodifica PCM sem tocar som. Ele não comprova a reprodução
  audível pelo FFplay nem a escolha correta da saída de áudio em outro computador.
- O diagnóstico de captura comprova leitura local, sem avaliar qualidade humana.
- Build Release, distribuição e execução no segundo PC ainda não foram validados.
- A proteção de helpers via Job Object está configurada e compilada; encerramento
  forçado pelo gerenciador de tarefas não foi ensaiado como teste separado.
- O funcionamento na LAN foi confirmado no teste informado. Alcance pela internet,
  regras da infraestrutura pública e a máquina da Etapa 2 ainda precisam de verificação.
- Os avisos de linker observados apenas informam a criação da biblioteca de
  importação Windows; não impediram a compilação.

A conclusão da Etapa 1 não comprova conectividade entre redes diferentes ou TURN.

## Etapa 2 — configuração local do cliente — 21/10/2026

- O responsável informou que está sem acesso à máquina de hospedagem e pediu
  continuidade sem ela. A implementação local avança; não se presume sistema,
  IP público, domínio ou acesso administrativo dessa máquina.
- Adicionados JSON local opcional (`CHAMADA_CONFIG`), STUN/TURN por UDP,
  retransmissão forçada e suporte WSS com raízes confiáveis do sistema.
- A API Flutter–Rust foi preservada. O evento adicional `caminho_ice` apresenta
  o tipo do par realmente selecionado, separado do estado da chamada.
- `cargo test -p chamada --lib`: quatro testes aprovados. Cobrem manutenção da
  configuração LAN, mapeamento TURN/Relay, rejeição de configuração inválida e
  exigência de TLS para token fora do loopback, sem expor os segredos nos erros.
- `dart format lib tool`: cinco arquivos verificados, sem alterações necessárias.
- `flutter analyze`: nenhum problema encontrado.
- Esses testes não exercitam um servidor TURN nem um endpoint WSS. A aceitação
  em redes diferentes continua pendente, inclusive chamada de cinco minutos,
  liberação dos dispositivos, repetição e ensaio com retransmissão forçada.
- `cargo test -p chamada --test chamada_sintetica -- --ignored --nocapture`:
  aprovado. Duas chamadas seguidas em loopback receberam vídeo remoto e PCM
  nos dois clientes, liberaram recursos e apresentaram `direto (local=host,
  remoto=host)` em ambos. Rodada 1: quadros `[5, 5]`, blocos PCM `[66, 69]`;
  rodada 2: quadros `[5, 14]`, blocos PCM `[83, 83]`. Sem captura real ou som.

## Etapa 2 — acesso à sinalização e pacote Windows — 21/10/2026

- `SINALIZACAO_TOKEN` habilita o controle de acesso antes da criação da sala;
  o endereço padrão com token é loopback. Sem a variável, o modo LAN foi mantido.
- `cargo test -p sinalizacao` encontrou o executável `target/debug/sinalizacao.exe`
  em uso (`Acesso negado`). O processo existente foi preservado. A alternativa
  `cargo test -p sinalizacao --target-dir target/verificacao-sinalizacao` passou:
  quatro testes de integração com WebSockets reais em loopback, cobrindo os
  fluxos LAN anteriores, rejeição de token ausente/incorreto, entrada e troca de
  SDP autorizadas, limpeza e rejeição de configuração com token vazio.
- Na repetição da suíte surgiu um reset TCP intermitente (`10054`) ao receber o
  aviso `saiu` após negociação autorizada. O rastreamento localizou a falha no
  encerramento da sala, não na verificação do token. O servidor agora reúne as
  metades do WebSocket e aguarda o fechamento por até um segundo antes de soltar
  o TCP; usa o mesmo cuidado nas rejeições de entrada. Após a correção, dez
  execuções seguidas dos quatro testes de integração passaram (40 verificações).
- A regressão sintética foi repetida com o encerramento corrigido: duas chamadas
  passaram, com caminho direto nos dois clientes. Rodada 1: quadros `[7, 10]`,
  PCM `[57, 56]`; rodada 2: quadros `[9, 9]`, PCM `[53, 53]`.
- `cargo fmt --all -- --check` e `git diff --check`: aprovados na revisão final.
- `cargo check --workspace`: aprovado para biblioteca cliente e servidor.
- `flutter build windows --debug`: aprovado, com o pacote atualizado em
  `app/build/windows/x64/runner/Debug/`. O aviso do linker sobre biblioteca de
  importação permanece informativo; não houve erro de compilação.
- O pacote foi compilado, sem nova execução visual, captura real ou aceitação
  humana nesta rodada. A confirmação humana da Etapa 1 refere-se à versão anterior.
- O guia da Etapa 2 registra configuração do cliente, token, proposta de proxy
  Caddy/coturn, portas, NAT e roteiro de aceitação. São instruções preparadas,
  não comandos executados na hospedagem. Nenhum serviço público foi exposto.
- Pendentes: inspeção da máquina, escolha e registro das versões de Caddy/coturn,
  implantação com certificado confiável, alcance público, áudio/vídeo entre redes
  e ensaio de mídia com retransmissão forçada. TURN TCP/TLS segue fora do escopo
  deste incremento; não há comprovação de funcionamento em redes que bloqueiem UDP.

## Revisão da abordagem de hospedagem — 22/10/2026

- O responsável substituiu o cenário Ubuntu remoto com VPN por um terceiro
  computador Windows doméstico, com IP privado e acesso ao roteador. Confirmou
  que os dois participantes acessarão pela internet, sem VPN.
- A documentação propõe sinalização Rust e Caddy nativos no terceiro Windows,
  além de coturn Windows com referência 4.18.0 e MSVC/vcpkg. O roteiro detalha
  verificação de CGNAT/duplo NAT, reserva DHCP, DNS/DDNS, regras TCP/UDP e WSS.
- O fluxo de compilação TURN foi conferido na fonte upstream da versão; não
  foi executado neste projeto. A configuração evita as opções `no-cli` e
  `no-dtls`, removidas nessa versão.
- Nesta revisão foram alterados apenas documentos. Não houve acesso ao terceiro
  Windows, alteração de roteador, compilação de dependências, emissão de
  certificado ou chamada externa. As evidências históricas acima permanecem.
- Pendentes: confirmar entrada IPv4 da residência, compilar/executar o pacote
  TURN Windows, implantar e verificar serviços e portas, executar os dois ensaios
  de chamada e conferir a inicialização após reboot caso seja configurada.

## Auditoria e adaptação IPv6 — 23/10/2026

Esta seção atualiza o estado da Etapa 2 e **não reescreve os testes anteriores**.
A entrada IPv4 ficou inviável sob CGNAT conforme investigação do responsável.
A topologia atual usa IPv6 global do Windows S, DNS AAAA, Caddy/WSS público e
coturn 4.18.0 nativo, sem VPN. Os IPs informados são observações daquela rede,
não constantes incorporadas ao projeto.

### Investigação e mudanças

- O caminho Flutter→FRB→Rust, o parser WSS, a sinalização/salas e os helpers
  locais já eram compatíveis com a separação de famílias. Foram preservados.
- O bind WebRTC restringia mídia a IPv4. Foi ampliado para wildcards IPv4/IPv6;
  a biblioteca expande cada um nas interfaces utilizáveis daquela família.
- A inspeção dos fontes locais fixados confirmou DNS IPv6 no runtime ICE,
  combinação de endereços da mesma família e suporte a candidatos host/srflx/relay.
  Não foi deduzido suporte completo a partir do nome da biblioteca.
- rtc-turn 0.21.1 não envia REQUESTED-ADDRESS-FAMILY. O coturn dedicado deve usar
  `allocation-default-address-family=ipv6`, opção conferida no parser 4.18.0.
- TURN IPv6 literal sofre reconstrução sem colchetes no webrtc 0.21.1. A validação
  agora o rejeita antecipadamente com orientação para hostname AAAA. Não há fork
  da biblioteca nem alteração do formato JSON ou das dependências do projeto.
- O diagnóstico do par selecionado agora inclui tipo, protocolo, família, IP e
  porta em ambos os candidatos. Os endereços ficam no diagnóstico local.
- Criados exemplos Caddy/coturn IPv6, reescrito integralmente o roteiro 07 e
  alinhados README, arquitetura, etapas, guia e contratos. Roteiro/relatório
  diferenciam configuração, compilação, transporte local e aceitação física.

### Resultados executados

| Verificação | Resultado e alcance |
| --- | --- |
| `cargo test --locked --workspace` | 14 testes aprovados: seis de configuração, três IPv6 e cinco de sinalização; dois ignorados por pré-requisitos próprios |
| Teste de parsing/regras IPv6 | WSS/literal com colchetes, loopback sem TLS, exigência TLS/token, STUN literal e rejeição explicada do TURN literal |
| Resolução ICE | Runtime da biblioteca retornou IPv6 para localhost; não é consulta AAAA pública |
| STUN UDP IPv6 | Servidor de teste com mensagens da biblioteca produziu candidato srflx IPv6 no SDP; endereço de documentação controlado, não observação de NAT |
| RTP IPv6 | Duas PeerConnections em ::1 negociaram trilhas VP8/Opus, receberam payloads conhecidos nas duas trilhas de ambos os lados, exigiram par host↔host IPv6 e repetiram após fechar |
| Sinalização IPv6 | WebSocket real em ::1, rejeição de token ausente/incorreto e negociação autorizada sem ocupar sala indevidamente |
| `cargo test --locked -p chamada --test chamada_sintetica -- --ignored --nocapture` | Aprovado: duas chamadas no caminho completo da aplicação com vídeo/áudio gerados e decodificados |
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --locked --workspace` | Aprovado |
| `cargo build --locked -p sinalizacao --release` | Aprovado |
| Inicialização da sinalização Release | Executável iniciou com token em 127.0.0.1:0 e [::1]:0; processos de teste encerrados |
| `cargo clippy --locked --workspace --all-targets` | Concluído com avisos de estilo preexistentes; ver observação abaixo |
| `dart format --output=none --set-exit-if-changed lib tool` | Cinco arquivos, nenhuma alteração |
| `dart run tool/verificar_ponte.dart` | Ponte, stream, erro assíncrono e encerramento aprovados com DLL atualizada |
| `flutter analyze` | Nenhum problema encontrado |
| `flutter build windows --release` | Aprovado; pacote completo em app/build/windows/x64/runner/Release |
| `git diff --check` | Sem erros de espaços em branco; Git informa conversão de fins de linha conforme atributos |
| Comandos da documentação | 39 blocos PowerShell analisados pelo parser, sem erro sintático; não executados |
| Links da documentação | Referências locais conferidas, nenhum destino ausente |

Na regressão sintética, o caminho escolhido continuou **host IPv4** nos dois
clientes. Rodada 1: quadros remotos `[11, 12]`, blocos PCM `[69, 69]`.
Rodada 2: quadros `[12, 11]`, PCM `[70, 68]`. Isso comprova regressão de mídia
local com o novo bind duplo, não decodificação audiovisual por IPv6 público.
O ensaio ::1 comprova transporte RTP por ICE/DTLS/SRTP IPv6, mas usa payloads
conhecidos; não mede qualidade nem produz som ou frames decodificados.

Clippy não estava instalado; o componente foi adicionado à toolchain usada na
verificação. A execução estrita inicial com `-D clippy::all` falhou em avisos
preexistentes de `collapsible_if`/`needless_update` em sinalização, chamada e
exemplo de captura. Eles foram preservados para evitar refatorações de código
compatível fora do escopo. Os dois avisos encontrados no teste novo foram
corrigidos e o teste foi repetido. A execução normal do Clippy concluiu; isso
não equivale a uma execução estrita sem avisos.

As primeiras execuções dos testes IPv6 expiraram sob a restrição de rede do
sandbox. Até uma troca UDP simples em ::1 expirou nesse ambiente. A mesma suíte
passou fora dessa restrição, sem alterar firewall real. Houve também uma falha
inicial de compilação por importação de trait no teste e correção do SSRC do
payload para o negociado; a versão final foi reexecutada com sucesso.

Flutter 3.47.6, Dart 3.13.5, Rust/Cargo 1.98.1 e alvo windows-x64 foram conferidos.
Doctor reconheceu Build Tools 2026/SDK Windows e indicou pendências Android/Chrome,
alvos fora do escopo. O SDK Flutter exigiu escrita no lock/cache fora do workspace;
a verificação foi repetida com esse acesso e passou. Avisos de linker sobre
biblioteca de importação e de cache incremental por cópia não impediram builds.

### Pendências e limites mantidos

- Coturn 4.18.0 nativo não foi compilado/executado nesta rodada. A release oficial
  consultada oferece fontes/atestado, sem pacote Windows pronto nos assets; o
  roteiro preserva build MSVC/vcpkg upstream. O teste TURN real de ipv6.rs está
  preparado e ignorado até existir processo coturn de teste isolado.
- O padrão de alocação IPv6 no coturn foi conferido em fonte, **não comprovado
  por alocação/relay real** neste ambiente. TURN UDP via hostname AAAA foi
  auditado, mas não exercitado externamente. TURN TCP/TLS segue fora da integração.
- Caddy/configuração foram documentados a partir de fontes oficiais; não houve
  instalação de Caddy em S, validação do Caddyfile com seu executável, emissão
  ACME, publicação AAAA, atualização DDNS nem mudança de firewall/roteador.
- O firmware FGA2110 real permanece por conferir. Não se presume suporte a menus
  ou regras específicas que não tenham sido inspecionados.
- Seguem pendentes os cinco cenários físicos: WSS IPv6 autenticado de A/B externos,
  mídia normal, mídia TURN forçado com relay↔relay IPv6/contadores, cinco minutos
  em cada modo e encerramento/repetição sem reiniciar serviços de S.
- Redes somente IPv4 não alcançam o serviço somente IPv6. Os três computadores
  precisam das condições de conectividade descritas no roteiro.
- Nenhuma aceitação visual, qualidade audível, captura real ou teste entre redes
  foi declarado nesta rodada. Nenhum commit foi realizado.

Consultar [auditoria](auditoria-ipv6.md) e [roteiro operacional](roteiro-entre-redes.md).

O diagnóstico e as correções após a primeira tentativa humana pela internet
estão no [registro da chamada](diagnostico-primeira-chamada.md).

## Reorganização documental — 27/10/2026

- Separados os procedimentos e registros em `docs/desenvolvimento/` e os
  estudos em `docs/estudos/`, com índice geral e mapa dos caminhos anteriores.
- Estudos divididos em arquitetura, WebRTC, mídia, servidor, ponte Flutter–Rust,
  Flutter e infraestrutura. Cada módulo relaciona funcionamento e arquivos.
- O módulo WebRTC explica a API 0.21.1, negociação, transporte RTP, candidatos,
  cancelamento, limitações confirmadas e soluções usadas na POC.
- Explicações conferidas com o código local, manifestos e testes. Os dois
  contornos TURN IPv6 foram reconferidos nos fontes locais de webrtc/rtc-turn
  0.21.1; a documentação pública dessa versão foi consultada para a arquitetura
  da biblioteca. Não houve atualização de dependências.
- Links locais, fechamento de blocos Markdown e sintaxe dos blocos PowerShell
  foram verificados. Os comandos documentados não foram executados nesta revisão.
- Nenhuma nova execução de build, chamada, captura real ou ensaio de rede foi
  realizada para esta mudança de documentação. Os critérios de aceite e os
  resultados históricos permanecem distintos dessa conferência documental.
