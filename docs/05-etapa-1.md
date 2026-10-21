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

As instruções de chamada serão acrescentadas junto à integração funcional.
