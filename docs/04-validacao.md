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

## Limites

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
