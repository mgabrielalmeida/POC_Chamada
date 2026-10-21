# POC de chamada 1:1 — Flutter, Rust e webrtc-rs

Experimento de chamada de áudio e vídeo entre **dois computadores Windows**.
O objetivo é avaliar `webrtc-rs` para um aplicativo maior, mantendo esta POC
pequena, legível e rápida de implementar.

Flutter apresenta a interface. Rust concentra mídia, rede e controle da chamada.
Flutter-Rust Bridge faz a comunicação entre essas camadas.

## Estado atual

Implementação da Etapa 1 disponível: ponte Flutter–Rust, sinalização por salas e caminho
de áudio e vídeo com `webrtc-rs`. A mídia usa FFmpeg auxiliar, controlado por Rust.
Os resultados e limites estão em [ambiente e validação](docs/04-validacao.md).
A aceitação da chamada requer dois computadores físicos.

## Estrutura

- `app/`: interface Flutter e executor nativo Windows.
- `rust/`: biblioteca técnica do cliente.
- `servidor/`: executável de sinalização em Rust.
- `docs/`: arquitetura, etapas, guia do código e evidências de validação.
- `Cargo.toml` e `Cargo.lock`: workspace Rust compartilhado.

## Preparar e executar

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

Na raiz, prepare a mídia e inicie a sinalização em um dos computadores:

```powershell
.\scripts\preparar-midia.ps1
cargo run -p sinalizacao
```

Em outro terminal, abra o aplicativo. Repita a preparação e a abertura do
aplicativo no segundo computador:

```powershell
cd app
flutter pub get
flutter run -d windows
```

Em ambos os aplicativos, informe `ws://IP_DO_SERVIDOR:8080` e o mesmo nome de sala.
Clique em **Criar / entrar**. A primeira pessoa aguarda a segunda. Use **Encerrar**
antes de iniciar outra chamada. Para os dois PCs, o IP deve ser o IPv4 LAN do
servidor, não `127.0.0.1`. Consulte o [guia da Etapa 1](docs/05-etapa-1.md) para
firewall, distribuição e diagnósticos. Esta etapa usa rede local confiável.

## Verificar mudanças

Na raiz:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo test -p sinalizacao
```

Na pasta `app/`:

```powershell
dart format --output=none --set-exit-if-changed lib tool
flutter analyze
flutter build windows --debug
```

Esses comandos verificam compilação e análise. A aceitação da chamada exigirá dois computadores
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
- **A outra pessoa não entra:** confira o IP LAN, a mesma sala, o servidor ativo
  e a liberação de TCP 8080 no firewall da rede privada.
- **Conecta mas não chega mídia:** confira permissões de câmera/microfone e
  liberação do aplicativo no firewall para UDP na rede privada.
- **FFmpeg ausente:** execute `scripts/preparar-midia.ps1` antes de compilar.

Problemas reais de mídia e rede serão documentados à medida que forem observados.

## Documentação e versionamento

- [Arquitetura e decisões](docs/01-arquitetura.md)
- [Etapas e critérios de conclusão](docs/02-etapas.md)
- [Guia didático do código](docs/03-guia-do-codigo.md)
- [Ambiente e validação](docs/04-validacao.md)
- [Execução e explicação da Etapa 1](docs/05-etapa-1.md)

O Git é local, com branch `main`, sem publicação remota. Use commits pequenos e
coerentes. Versione os lockfiles; não versione saídas de compilação, segredos ou
arquivos de ambiente locais. Documentação, comentários próprios e identificadores
que não dependam de contratos externos devem usar português.
