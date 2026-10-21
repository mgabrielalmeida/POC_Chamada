# Ambiente e validação

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

## Limites atuais da Etapa 1

- Pendente: chamada de cinco minutos em dois computadores físicos na mesma rede,
  qualidade audível/visual, encerramento e nova chamada nesse cenário.
- O teste sintético decodifica PCM sem tocar som. Ele não comprova a reprodução
  audível pelo FFplay nem a escolha correta da saída de áudio em outro computador.
- O diagnóstico de captura comprova leitura local, sem avaliar qualidade humana.
- Build Release, distribuição e execução no segundo PC ainda não foram validados.
- A proteção de helpers via Job Object está configurada e compilada; encerramento
  forçado pelo gerenciador de tarefas não foi ensaiado como teste separado.
- Firewall, alcance LAN e a máquina da Etapa 2 ainda precisam de verificação real.
- Os avisos de linker observados apenas informam a criação da biblioteca de
  importação Windows; não impediram a compilação.

A implementação não encerra a aceitação da etapa. Registre aqui os resultados
dos dois dispositivos antes de considerar o objetivo 1 concluído.
