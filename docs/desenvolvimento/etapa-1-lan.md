# Etapa 1 — preparar e executar na LAN

[Desenvolvimento](README.md) · [Critérios de aceite](etapas.md) · [Testes](testes.md)

Este guia reúne comandos e condições de execução. Para entender as tecnologias,
leia os módulos de [ponte](../estudos/ponte-flutter-rust/README.md),
[mídia](../estudos/midia/README.md), [servidor](../estudos/servidor/README.md) e
[WebRTC](../estudos/webrtc/README.md).

## 1. Conferir o ambiente

São necessários Flutter com alvo Windows, Rust MSVC e Visual Studio Build Tools
com C++ desktop e Windows SDK. Em PowerShell, na raiz do checkout:

```powershell
flutter --version
flutter doctor -v
flutter devices
rustc --version
cargo --version
```

O alvo esperado é windows-x64. Pendências Android/Chrome não são requisitos
desta POC. As versões efetivamente observadas nas rodadas anteriores estão em
[validação](validacao.md); confira as instaladas na máquina do novo ensaio.

## 2. Preparar a mídia

```powershell
.\scripts\preparar-midia.ps1
```

O script baixa FFmpeg 9.0.2 essentials, confere SHA-256 e extrai em `.local/`.
O download de referência tem cerca de 109 MB. Não altera o PATH e não instala
serviços. O build copiará FFmpeg, FFplay e licenças para o pacote do app.
O [estudo do script](../estudos/midia/arquivos.md) explica seus limites.

O hash esperado no script é
`60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba`, correspondente
à [referência publicada da distribuição fixada](https://www.gyan.dev/ffmpeg/builds/packages/ffmpeg-9.0.2-essentials_build.zip.sha256).
Se não conferir, interrompa a preparação e investigue a origem do arquivo.

## 3. Preparar a ponte

Os bindings são versionados. Para uma execução comum, use os arquivos existentes.
Se mudou a API Rust ou precisa regenerar, use a versão **2.13.0**, igual ao crate
Rust e ao pacote Dart. Na raiz:

```powershell
cargo install flutter_rust_bridge_codegen --version 2.13.0 --locked
cargo install cargo-expand --locked
Push-Location app
flutter pub get
flutter_rust_bridge_codegen generate
Pop-Location
```

O gerador também pode instalar cargo-expand quando ausente. Não edite
`frb_generated*` nem `app/lib/src/rust/` manualmente. Depois, confira a travessia:

```powershell
cargo build --locked -p chamada
Push-Location app
dart run tool/verificar_ponte.dart
Pop-Location
```

O utilitário confirma mensagem, stream, erro assíncrono e encerramento, sem
usar câmera, microfone ou servidor. Feche-o e feche o aplicativo antes de
recompilar a DLL no Windows. `app/windows/rust.cmake` integra o build Cargo
à compilação do aplicativo.

## 4. Iniciar a sinalização em um computador

Na raiz do checkout escolhido para hospedar a sala:

```powershell
cargo run -p sinalizacao
```

Sem `SINALIZACAO_TOKEN`, o padrão é `0.0.0.0:8080`. Para um ensaio restrito à
própria máquina, o endereço pode ser escolhido explicitamente:

```powershell
cargo run -p sinalizacao -- 127.0.0.1:8081
```

O segundo comando não atende os outros computadores. Para a chamada LAN,
use a escuta acessível pela interface local e identifique seu IPv4 com:

```powershell
ipconfig
```

O servidor é um processo separado; mantenha seu terminal aberto. O modo WS
sem autenticação destina-se à LAN confiável. Para internet, siga
[Etapa 2](etapa-2-configuracao.md), com token e WSS.

## 5. Abrir os dois clientes

Em outro terminal, na raiz de cada checkout que será compilado:

```powershell
Push-Location app
flutter pub get
flutter run -d windows
Pop-Location
```

Nos dois aplicativos, preencha `ws://IP_ANOTADO:8080`, substituindo
`IP_ANOTADO` pelo IPv4 LAN do servidor, e use a mesma sala, por exemplo `teste`.
Não use `127.0.0.1` no cliente que está em outro computador.
Clique em **Criar / entrar**; a primeira pessoa aguarda a segunda.

A sala aceita de um a 48 caracteres entre letras ASCII, números, hífen e
sublinhado. Tem no máximo duas pessoas. Ao terminar, ambos precisam entrar
novamente. O [estudo do servidor](../estudos/servidor/README.md) explica o protocolo.

Se o firewall solicitar, permita acesso na rede privada usada no teste.
A sinalização usa TCP 8080 por padrão; WebRTC usa UDP em portas escolhidas
pelo sistema. A POC não altera firewall automaticamente. Isolamento entre
clientes Wi-Fi pode impedir a comunicação mesmo na mesma rede nominal.

Se estiver retomando um terminal da Etapa 2, confira as variáveis de ambiente:
`CHAMADA_CONFIG` altera a rede do cliente e `SINALIZACAO_TOKEN` altera o servidor.
Para um ensaio LAN sem essas configurações, use terminais sem essas variáveis.

## 6. Verificar mídia e encerramento

Use fones para evitar realimentação. Confira áudio e vídeo nos dois sentidos
por cinco minutos, clique em **Encerrar**, confirme liberação dos dispositivos
e repita a entrada dos dois computadores. Registre o resultado em
[validação](validacao.md), sem inferir duração que não tenha sido medida.

A POC usa a primeira câmera e o primeiro microfone DirectShow, sem seleção
na tela. Isso pode diferir do dispositivo preferido do Windows. No ensaio
histórico foram detectados ACER HD User Facing e Microphone Array (AMD Audio
Device); outros computadores podem ter nomes e ordem diferentes.

Não abra duas capturas da mesma câmera para simular dois PCs: o driver pode
permitir apenas um usuário. Para separar transporte desse limite, use os
ensaios do [guia de testes](testes.md):

```powershell
cargo run -p chamada --example diagnosticar_midia
cargo test -p chamada --test chamada_sintetica -- --ignored --nocapture
cargo test -p sinalizacao
```

O primeiro usa dispositivos reais isoladamente. O segundo usa fontes sintéticas,
sem captura ou som, e requer os helpers. O terceiro verifica o protocolo de
salas. Consulte os pré-requisitos e limites antes de interpretar o resultado.

## 7. Distribuir sem toolchains no segundo PC

Depois de preparar a mídia:

```powershell
Push-Location app
flutter build windows --release
Pop-Location
cargo build --locked -p sinalizacao --release
```

Copie a pasta inteira `app/build/windows/x64/runner/Release/`: executável,
DLLs, `data/`, FFmpeg, FFplay e `licencas/`. O destino precisa do runtime
Microsoft Visual C++ x64 compatível. Preserve licenças e condições de
redistribuição da distribuição FFmpeg utilizada.

O servidor separado é `target/release/sinalizacao.exe`. Não é necessário
rodá-lo em todos os participantes: ele deve estar na máquina que hospeda a sala.

Registre build Release e execução no segundo PC quando realizados. Um build
Debug local não comprova nenhum desses dois passos. Para investigação,
comece por captura; se funcionar, confira servidor/endereço/sala e depois
transporte UDP e par ICE. Os estudos explicam [mídia](../estudos/midia/README.md)
e [WebRTC](../estudos/webrtc/README.md) para orientar essa separação.
