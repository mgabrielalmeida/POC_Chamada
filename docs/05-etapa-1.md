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

A saída inclui `Ponte Flutter–Rust conectada.` e a confirmação de stream, erro
assíncrono e encerramento. O teste não usa câmera, microfone, servidor ou rede.
Feche o utilitário antes de recompilar a DLL no Windows. A compilação incorpora a DLL através do
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

Este procedimento usa `ws://` para teste em rede local confiável, sem autenticação.
Não exponha esse modo à internet. A preparação de token e WSS está no
[guia da Etapa 2](06-etapa-2.md).
Verifique o servidor com `cargo test -p sinalizacao`.

## Mídia: uma escolha para chegar rápido ao experimento

`webrtc` e `rtc` foram fixados em **0.21.1**. Não usamos o WebRTC do Flutter nem
LiveKit. A conexão, ICE, DTLS e SRTP passam pelo webrtc-rs na camada Rust.

FFmpeg **9.0.2 essentials**, distribuição Windows de gyan.dev indicada na página
do FFmpeg, faz captura DirectShow, VP8 e Opus. O script de preparação baixa uma
versão fixa e verifica o [SHA256 publicado](https://www.gyan.dev/ffmpeg/builds/packages/ffmpeg-9.0.2-essentials_build.zip.sha256):
`60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba`.
O arquivo e a extração ficam em `.local/`, fora do Git. CMake copia os executáveis
e a licença junto à aplicação. Não é necessário alterar o PATH do Windows.

Rust controla quatro processos por chamada:

| Processo | Trabalho |
| --- | --- |
| FFmpeg da câmera | Captura vídeo, produz RGBA local e codifica VP8 em RTP. |
| FFmpeg do microfone | Captura áudio e codifica Opus em RTP. |
| FFmpeg de vídeo remoto | Decodifica VP8 recebido e entrega RGBA a Rust. |
| FFplay de áudio remoto | Decodifica Opus e reproduz na saída padrão do Windows, sem janela. |

Os sockets RTP auxiliares escutam somente em `127.0.0.1`, com portas escolhidas
localmente. A rede entre computadores segue por WebRTC; os helpers não enviam
mídia diretamente ao outro PC. Arquivos SDP locais temporários descrevem os
codecs aos decodificadores e são removidos ao encerrar.

O vídeo é 320×240 a 10 fps, cerca de 300 kbit/s; áudio Opus usa 48 kHz, dois canais
e cerca de 32 kbit/s. Bitrates são configurações dos encoders, não medições de
tráfego total. Quadros atrasados são descartados para evitar filas crescentes.
Flutter apenas converte RGBA em imagem visível.

Para evitar outra integração nativa de codecs neste prazo, aceitamos processos
auxiliares e buffers via loopback. Isso aumenta o pacote e o consumo de recursos.
A POC avalia o caminho WebRTC; não demonstra que essa estratégia de captura e
codecs seja ideal para o aplicativo maior. Não há cancelamento de eco, controle
adaptativo de bitrate ou sincronização avançada entre áudio e vídeo. Use fones
nos testes para evitar realimentação sonora.

## Executar em dois computadores Windows

1. Em ambos, execute `.\scripts\preparar-midia.ps1` na raiz. O download é de cerca
   de 109 MB; compilação e uso não dependem de serviços externos de comunicação.
2. No computador que hospedará a sala, execute `cargo run -p sinalizacao`.
3. Use `ipconfig` nesse computador e anote o IPv4 da interface da rede local.
4. Em ambos, na pasta `app`, execute `flutter pub get` e `flutter run -d windows`.
5. Nas duas janelas, use `ws://IP_ANOTADO:8080` e a mesma sala, por exemplo `teste`.
6. Clique em **Criar / entrar**. A chamada começa quando a segunda pessoa entra.
7. Confira áudio e vídeo nos dois sentidos por cinco minutos. Depois clique em
   **Encerrar** e repita a entrada nos dois computadores.

Se o firewall solicitar acesso, permita servidor e aplicativo somente na rede
privada usada no teste. Sinalização usa TCP 8080 por padrão; WebRTC usa UDP em
portas escolhidas pelo sistema. O projeto não altera regras de firewall sozinho.
Redes com isolamento entre clientes Wi-Fi podem impedir o teste.

Não abra duas capturas da mesma câmera para simular dois dispositivos: o driver
pode aceitar só um usuário. O teste sintético abaixo separa o transporte desse
limite. Nesta máquina, foram detectados ACER HD User Facing e Microphone Array
(AMD Audio Device). Outros PCs podem enumerar os dispositivos em ordem diferente.
Usamos a primeira câmera e o primeiro microfone DirectShow; isso pode diferir do
microfone preferido nas configurações do Windows. Não há seleção na interface.

## Distribuir sem instalar ferramentas no segundo PC

Após preparar a mídia, execute `flutter build windows --release` em `app`.
Copie **a pasta inteira** `app/build/windows/x64/runner/Release/`, incluindo DLLs,
`data/`, `ffmpeg.exe`, `ffplay.exe` e `licencas/`. O destino precisa do runtime
Microsoft Visual C++ x64 compatível. A licença copiada e as condições da
distribuição FFmpeg devem acompanhar qualquer redistribuição.

O servidor é separado: compile com `cargo build -p sinalizacao --release` na raiz
e execute `target/release/sinalizacao.exe` no PC que hospedará a sala.
Build Release e execução no segundo PC devem ser registrados quando testados;
um build Debug local não os comprova.

## Diagnósticos reproduzíveis

Na raiz:

```powershell
cargo run -p chamada --example diagnosticar_midia
cargo test -p chamada --test chamada_sintetica -- --ignored --nocapture
```

O primeiro abre dispositivos reais, lê um quadro RGBA e um pacote Opus RTP,
depois libera tudo. Não verifica qualidade audível ou chamada remota.

O segundo inicia sinalização em loopback e duas sessões Rust com vídeo de teste
e tom gerado, sem capturar dispositivos nem reproduzir som. Exige quadros remotos
decodificados e blocos PCM remotos em ambos, encerra e repete a chamada. É ignorado
no teste comum porque depende dos executáveis auxiliares. Não substitui aceitação
em dois computadores físicos ou em redes diferentes.

Para investigar falhas, comece pelo diagnóstico de mídia. Se passar, confira
servidor, endereço, sala e firewall. Encerrar aguarda Rust fechar WebRTC e esperar
os helpers terminarem. Um Job Object do Windows também associa os helpers à
sessão, encerrando-os se a janela ou o processo principal morrer abruptamente.
