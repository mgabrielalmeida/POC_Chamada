# Mídia: dos dispositivos aos pacotes e de volta

[Trilha de estudos](../README.md) · [Arquivo a arquivo](arquivos.md)

Um quadro da câmera e um pacote WebRTC não são a mesma coisa. A câmera entrega
imagens; o codec as comprime; RTP organiza a mídia em pacotes; a pilha WebRTC
negocia e protege o transporte. No outro lado, é preciso percorrer o caminho
inverso até os pixels e o áudio reproduzido.

A POC delega captura e codecs a FFmpeg/FFplay e mantém sua coordenação em Rust.
Os processos auxiliares ficam invisíveis. Só o Flutter possui a janela do app.

## Os quatro helpers de uma chamada real

| Helper | Entrada | Saída |
| --- | --- | --- |
| Câmera — FFmpeg | Primeiro dispositivo de vídeo DirectShow | RGBA local no stdout e VP8/RTP por UDP local |
| Microfone — FFmpeg | Primeiro dispositivo de áudio DirectShow | Opus/RTP por UDP local |
| Receptor de vídeo — FFmpeg | RTP local que Rust retirou da trilha remota | RGBA remoto no stdout |
| Receptor de áudio — FFplay | RTP local que Rust retirou da trilha remota | Som na saída padrão do Windows |

Há duas saídas da câmera porque a prévia local não precisa esperar o quadro
ir ao outro computador e voltar. Uma saída é preparada para apresentação;
a outra passa pelo encoder VP8 para transmissão. Espelhar a prévia na tela
não altera o que o encoder envia.

No modo sintético, câmera e microfone são substituídos por `testsrc2` e um tom
de 440 Hz. O receptor de áudio passa a ser FFmpeg, produzindo PCM no stdout,
sem tocar som. Assim o teste pode verificar decodificação sem depender de
dispositivos físicos ou avaliação auditiva.

## Onde circulam RTP e pixels

Na saída, o helper envia RTP para um socket Rust em `127.0.0.1`.
`enviar_rtp`, de `chamada.rs`, adapta o SSRC e entrega à trilha WebRTC.
Na chegada, `receber_rtp` retira o pacote da trilha, adapta o cabeçalho e o
encaminha ao decoder local. Não há envio de RTP auxiliar diretamente a A ou B.

Os pixels RGBA seguem outro caminho: stdout do helper → leitura Rust →
`EventoMidia::Quadro` → `EventoChamada` → FRB → Flutter.
Cada quadro tem 320 × 240 × 4 = **307.200 bytes**. A 10 fps, são aproximadamente
3,07 MB/s por sequência de quadros antes de considerar cópias e descarte. Isso
é volume bruto interno de pixels, não o bitrate de rede do vídeo VP8.

O encoder é configurado para cerca de 300 kbit/s de vídeo e 32 kbit/s de áudio.
Esses são alvos do encoder, não medições do tráfego final com cabeçalhos,
criptografia e eventual retransmissão.

## Por que existem arquivos SDP locais

FFmpeg/FFplay precisam saber em qual porta receber e como interpretar os
payloads. Rust gera `video.sdp` e `audio.sdp` numa pasta temporária, com
loopback, porta, payload type, codec e direção `recvonly`.

Esses arquivos não são a oferta/resposta do WebRTC. O decoder não participa
de ICE, DTLS ou SRTP. Ele recebe RTP já retirado da conexão pela biblioteca,
com o contrato local adaptado. Os arquivos temporários somem quando seu
proprietário `TempDir` é liberado.

## Prontidão, filas e vida dos processos

Os receptores são iniciados antes dos capturadores e da conexão. Isso reduz
o risco de perder os primeiros pacotes úteis de vídeo. Mas um processo iniciado
não equivale a uma confirmação explícita de que abriu sua porta: a POC não
implementa um protocolo de prontidão com o helper.

As leituras de quadros usam canais pequenos e `try_send`. Se a interface não
acompanhar, um quadro é descartado em vez de virar atraso acumulado. O descarte
nesses canais afeta apresentação local/remota; não é um controlador de
congestionamento dos pacotes enviados pelo WebRTC.

Os capturadores começam antes da conexão, mas o RTP deles só é útil depois de
`Connected`. Até lá, `enviar_rtp` lê e descarta os pacotes. Sem essa leitura,
o socket de loopback acumulava a captura da negociação e a entregava de uma
vez: o `ffplay` remoto reproduz áudio em tempo real, então o acúmulo virava
atraso permanente. O vídeo se recuperava porque seu receptor não espera o
relógio de reprodução.

Rust verifica a cada segundo se algum helper terminou. Nome, código de saída
e últimas linhas de stderr ajudam a distinguir câmera, microfone e receptores.
O [problema de timeout dos receptores](../webrtc/dificuldades-e-decisoes.md)
mostrou por que um erro genérico de FFmpeg era insuficiente.

No encerramento normal, Rust mata e aguarda os filhos e termina as tarefas de
leitura. `kill_on_drop` protege a liberação dos objetos Rust; o Job Object Windows
protege também quando o processo principal desaparece abruptamente.

## O que este caminho ainda não resolve

A primeira câmera/microfone enumerados podem não ser os dispositivos preferidos
do usuário. Não há seletor, troca durante a chamada, tratamento avançado de eco
ou medição de sincronismo audiovisual. Um pacote Opus válido demonstra captura
e codificação, mas não demonstra volume adequado, ausência de ruído ou qualidade
na outra máquina. Os [testes](../../desenvolvimento/testes.md) separam essas evidências.
