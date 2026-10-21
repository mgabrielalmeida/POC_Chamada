# Etapa 1 — implementação e execução

## Ponte Flutter–Rust

Foi fixada a versão **2.13.0** no crate Rust, no pacote Dart e no gerador.
A API manual fica em `rust/src/api.rs`. Os arquivos `frb_generated*` e os arquivos
em `app/lib/src/rust/` são gerados: não os edite diretamente.

Para instalar as ferramentas e regenerar, na raiz:

```powershell
cargo install flutter_rust_bridge_codegen --version 2.13.0 --locked
cargo install cargo-expand --locked
cd app
flutter pub get
flutter_rust_bridge_codegen generate
```

O gerador pode instalar `cargo-expand` automaticamente se ele estiver ausente.
Para verificar a travessia real Dart → DLL Rust, na raiz:

```powershell
cargo build -p chamada
cd app
dart run tool/verificar_ponte.dart
```

A saída esperada é `Ponte Flutter–Rust conectada.`. Esse teste não usa câmera,
microfone, servidor ou rede. A compilação Windows incorpora a DLL através do
arquivo curto `app/windows/rust.cmake`, sem pacote auxiliar de compilação.

## Fontes técnicas consultadas

- [Configuração do gerador FRB](https://cjycode.com/flutter_rust_bridge/guides/custom/codegen/inputs)
- [Exemplos oficiais webrtc-rs](https://github.com/webrtc-rs/webrtc/tree/master/examples)
- [Dispositivos DirectShow no FFmpeg](https://ffmpeg.org/ffmpeg-devices.html#dshow)
- [Distribuições Windows indicadas pelo FFmpeg](https://ffmpeg.org/download.html#build-windows)

## Sinalização LAN

Na raiz, `cargo run -p sinalizacao` inicia o servidor em `0.0.0.0:8080`.
Para escolher outro endereço: `cargo run -p sinalizacao -- 127.0.0.1:8081`.
O aplicativo usará `ws://IP_DO_COMPUTADOR:8080`. Uma sala admite duas pessoas.

O protocolo usa JSON por WebSocket:

- `entrar`, com `sala`: primeira mensagem de cada cliente.
- `aguardando`: a primeira pessoa aguarda a segunda.
- `pronto`, com `ofertante`: define quem cria a oferta para evitar colisão.
- `descricao`, com `descricao`: encaminha a oferta ou a resposta SDP completa.
- `saiu`: o outro participante desconectou; a sala é removida.
- `erro`, com `mensagem`: informa sala inválida ou lotada.

O nome da sala aceita até 48 letras ASCII, números, `-` ou `_`. A sala nasce
quando a primeira pessoa entra e não persiste após o encerramento. Para uma
nova chamada, ambos entram novamente. O servidor verifica resposta a pings e
limita mensagens a 64 KiB. Não usa banco de dados nem transmite mídia.

Esta versão usa `ws://` para teste em rede local confiável, sem autenticação.
Não exponha esse servidor à internet; a Etapa 2 tratará o acesso externo.
Verifique o servidor com `cargo test -p sinalizacao`.

As instruções de chamada serão acrescentadas junto à integração funcional.
