# POC de chamada 1:1 — Flutter, Rust e webrtc-rs

Experimento de chamada de áudio e vídeo entre **dois computadores Windows**.
O objetivo é avaliar `webrtc-rs` para um aplicativo maior, mantendo esta POC
pequena, legível e rápida de implementar.

Flutter apresenta a interface. Rust concentra mídia, rede e controle da chamada.
Flutter-Rust Bridge fará a comunicação entre essas camadas.

## Estado atual

Base inicial: aplicativo Flutter Windows com tela estática, biblioteca Rust e
executável Rust de sinalização. **Ainda não há ponte Flutter–Rust, captura de
mídia, webrtc-rs integrado, servidor escutando ou chamada funcional.** As versões
dessas dependências serão escolhidas e registradas quando forem integradas.

## Estrutura

- `app/`: interface Flutter e executor nativo Windows.
- `rust/`: biblioteca técnica do cliente.
- `servidor/`: executável de sinalização em Rust.
- `docs/`: arquitetura, etapas, guia do código e evidências de validação.
- `Cargo.toml` e `Cargo.lock`: workspace Rust compartilhado.

## Preparar e executar a base

Use PowerShell na raiz do repositório. São necessários Flutter com suporte a
Windows, Rust com toolchain MSVC e Visual Studio Build Tools com as ferramentas
C++ de desktop e Windows SDK. Confira o ambiente:

```powershell
flutter --version
flutter doctor -v
flutter devices
rustc --version
cargo --version
```

Para abrir a tela inicial:

```powershell
cd app
flutter pub get
flutter run -d windows
```

Em outro terminal, na raiz, para conferir os esqueletos Rust:

```powershell
cargo check --workspace
cargo run -p sinalizacao
```

O último comando apenas informa que o servidor ainda não foi implementado e
encerra. A tela Flutter funciona independentemente da biblioteca Rust nesta base.
Não há configuração de sala, endereço ou credenciais nesta etapa.

## Verificar mudanças

Na raiz:

```powershell
cargo fmt --all -- --check
cargo check --workspace
```

Na pasta `app/`:

```powershell
dart format --output=none --set-exit-if-changed lib
flutter analyze
flutter build windows --debug
```

Esses comandos verificam a base. A aceitação da chamada exigirá dois computadores
físicos e observação humana, conforme o [plano](docs/02-etapas.md).

## Solução de problemas

- **Flutter não consegue acessar o cache do SDK:** confira a permissão de escrita
  da instalação do Flutter e se outro processo está usando o SDK.
- **Windows não aparece como alvo:** confira `flutter devices` e, se necessário,
  execute `flutter config --enable-windows-desktop`.
- **Compilação Windows acusa ferramentas ausentes:** execute `flutter doctor -v`
  e instale os componentes C++ de desktop e Windows SDK indicados.
- **Plugins exigem links simbólicos:** habilite o Modo de Desenvolvedor do Windows
  e execute novamente a preparação do aplicativo.
- **A tela abre, mas não faz chamadas:** esse é o comportamento desta base inicial.

Problemas reais de mídia e rede serão documentados à medida que forem observados.

## Documentação e versionamento

- [Arquitetura e decisões](docs/01-arquitetura.md)
- [Etapas e critérios de conclusão](docs/02-etapas.md)
- [Guia didático do código](docs/03-guia-do-codigo.md)
- [Ambiente e validação](docs/04-validacao.md)

O Git é local, com branch `main`, sem publicação remota. Use commits pequenos e
coerentes. Versione os lockfiles; não versione saídas de compilação, segredos ou
arquivos de ambiente locais. Documentação, comentários próprios e identificadores
que não dependam de contratos externos devem usar português.
