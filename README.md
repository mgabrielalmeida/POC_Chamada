# POC de chamada 1:1 — Flutter, Rust e webrtc-rs

Experimento de chamada de áudio e vídeo entre **dois computadores Windows**.
O objetivo é avaliar `webrtc-rs` para um aplicativo maior, mantendo esta POC
pequena, legível e rápida de implementar.

Flutter apresenta a interface. Rust concentra mídia, rede e controle da chamada.
Flutter-Rust Bridge faz a comunicação entre essas camadas.

## Estado atual

**Etapa 1 concluída e testada em 21/10/2026:** funcionamento confirmado pelo
responsável em dois dispositivos Windows na mesma rede Wi-Fi. A ponte
Flutter–Rust, a sinalização por salas e áudio/vídeo usam `webrtc-rs`, com FFmpeg
auxiliar controlado por Rust. A Etapa 2, entre redes diferentes, está em implementação.
Os resultados e limites estão em [ambiente e validação](docs/desenvolvimento/validacao.md).

A preparação da Etapa 2 inclui configuração local de STUN/TURN, modo de
retransmissão forçada, diagnóstico do caminho ICE e sinalização com token para
uso atrás de WSS. A hospedagem escolhida é um terceiro Windows doméstico, com
IPv6 público e acesso pela internet, sem VPN, diante do CGNAT IPv4. O [roteiro detalhado](docs/desenvolvimento/roteiro-entre-redes.md)
explica seleção do IPv6, DNS AAAA, Caddy, coturn nativo e firewalls IPv6. O
[diagnóstico de 23/10](docs/desenvolvimento/diagnostico-primeira-chamada.md) registra
WSS acessível na tentativa humana e ensaios sintéticos por TURN no próprio S.
O aceite físico completo entre redes A/B continua pendente. A/B precisam de IPv6.
O cliente abre sockets IPv4/IPv6 e mostra o par ICE com família, endereço e tipo.
As limitações da biblioteca estão na [auditoria IPv6](docs/desenvolvimento/auditoria-ipv6.md).

## Estrutura

- `app/`: interface Flutter e executor nativo Windows.
- `rust/`: biblioteca técnica do cliente.
- `servidor/`: executável de sinalização em Rust.
- `docs/desenvolvimento/`: preparação, etapas, execução, testes e evidências.
- `docs/estudos/`: módulos didáticos das tecnologias e do código, com foco em webrtc-rs.
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
servidor, não `127.0.0.1`. Consulte o [guia da Etapa 1](docs/desenvolvimento/etapa-1-lan.md) para
firewall, distribuição e diagnósticos. Esta etapa usa rede local confiável.

## Verificar mudanças

Na raiz:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo test --locked --workspace
```

Na pasta `app/`:

```powershell
dart format --output=none --set-exit-if-changed lib tool
flutter analyze
flutter build windows --debug
```

Esses comandos verificam formatação, compilação, análise e testes locais.
O [guia de testes](docs/desenvolvimento/testes.md) separa os ensaios de ponte,
captura, transporte IPv6 e decodificação sintética. A aceitação da Etapa 1 foi
informada pelo responsável; a Etapa 2 exige nova observação humana em redes
diferentes, conforme o [plano](docs/desenvolvimento/etapas.md).

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

As falhas investigadas estão no [histórico de validação](docs/desenvolvimento/validacao.md)
e no [diagnóstico da primeira chamada](docs/desenvolvimento/diagnostico-primeira-chamada.md).

## Documentação e versionamento

Comece pelo [índice da documentação](docs/README.md). Há dois percursos:

- [Desenvolvimento e testes](docs/desenvolvimento/README.md): ambiente, execução
  LAN/internet, critérios de aceite, validação e investigações datadas.
- [Estudos](docs/estudos/README.md): arquitetura, WebRTC, mídia, servidor, ponte,
  Flutter e infraestrutura, com funcionamento geral e leitura arquivo a arquivo.

O [módulo de webrtc-rs](docs/estudos/webrtc/README.md) é o centro da trilha:
explica protocolos, negociação, RTP, uso da versão 0.21.1, dificuldades,
soluções adotadas e limites ainda presentes.

O Git é local, com branch `main`, sem publicação remota. Use commits pequenos e
coerentes. Versione os lockfiles; não versione saídas de compilação, segredos ou
arquivos de ambiente locais. Documentação, comentários próprios e identificadores
que não dependam de contratos externos devem usar português.
