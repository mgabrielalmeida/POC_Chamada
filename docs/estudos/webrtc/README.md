# WebRTC em Rust: o núcleo da POC

[Trilha de estudos](../README.md) · [Negociação](negociacao-e-transporte.md) ·
[Arquivos](arquivos.md) · [Dificuldades](dificuldades-e-decisoes.md)

WebRTC reúne protocolos que permitem negociar e transportar mídia em tempo
real. Nesta POC, ele precisa ligar dois processos Rust em computadores distintos,
apesar de diferentes interfaces, famílias IP, firewalls e possíveis NATs.
Não basta enviar os bytes da câmera para um endereço conhecido: os participantes
precisam concordar sobre o formato da mídia, encontrar um caminho utilizável
e proteger o tráfego.

O objeto que coordena isso é a **PeerConnection**. Cada sessão possui uma.
Ela conhece a descrição local, a descrição remota, as trilhas e os transportes.
Rust usa seus métodos para negociar e recebe eventos quando o estado muda ou
uma trilha remota aparece.

## webrtc-rs, webrtc e rtc

`webrtc-rs` é o projeto/ecossistema. No manifesto desta POC, os crates `webrtc`
e `rtc` estão fixados em `=0.21.1`. É importante estudar essa API específica:
exemplos de outras versões podem usar construtores, callbacks e tipos diferentes.

Na versão 0.21.1, `webrtc` oferece a interface assíncrona e um driver de fundo
que coordena sockets, temporizadores e eventos. Por baixo está o núcleo de
protocolos `rtc`, organizado no modelo **Sans-I/O**: a lógica de protocolo é
separada da execução concreta de entrada e saída. O runtime padrão de `webrtc`
é Tokio. A POC chama a camada assíncrona e importa de `rtc` tipos de codecs,
RTP e candidatos; não implementa manualmente o laço de I/O desse núcleo.
Veja a [documentação da versão 0.21.1](https://docs.rs/webrtc/0.21.1/webrtc/).

Isso explica a coexistência dos imports em `chamada.rs`: `PeerConnectionBuilder`
vem de `webrtc`, enquanto `RTCRtpCodec` e a estrutura do pacote RTP vêm de `rtc`.
Não são duas conexões concorrentes nem duas implementações escolhidas em tempo
de execução. São partes da mesma integração.

## O vocabulário, ligado ao código

| Conceito | Explicação simples | Onde aparece na POC |
| --- | --- | --- |
| Sinalização | Meio usado para os participantes combinarem a chamada | JSON por WebSocket; servidor de salas |
| SDP | Descrição textual dos parâmetros da sessão; não contém os quadros de vídeo | `RTCSessionDescription` nas mensagens `descricao` |
| Oferta e resposta | Uma proposta de sessão e a resposta do outro participante | `create_offer`, `create_answer`, `set_*_description` |
| ICE | Reúne possíveis caminhos e verifica quais pares conseguem se comunicar | Configuração, coleta, transporte e par selecionado |
| STUN | Ajuda a descobrir o endereço pelo qual um servidor observa o cliente | URLs `stun:` e candidatos `srflx` |
| TURN | Fornece um endereço em um servidor para retransmitir tráfego | URLs `turn:` e candidatos `relay` |
| DTLS | Handshake de segurança usado no transporte WebRTC | Executado pela biblioteca; a POC não escreve o handshake |
| SRTP | Proteção dos pacotes de mídia no transporte | Aplicada e removida pela pilha WebRTC |
| RTP | Pacotes com mídia codificada, sequência, timestamp e identificação da fonte | Entrada/saída das trilhas e sockets dos helpers |
| RTCP | Informações de controle associadas ao transporte RTP | Pilha/interceptores; portas auxiliares locais reservadas |
| Trilha | Fluxo lógico de áudio ou vídeo | Uma trilha local de cada tipo e as trilhas remotas recebidas |
| SSRC | Identificador da fonte de pacotes RTP | Gerado ao criar a trilha e ajustado no RTP de saída |
| Payload type | Número que identifica o formato negociado do payload RTP | VP8 com 96 e Opus com 111 nos contratos locais |

Não é preciso decorar as siglas de uma vez. Ao ler a negociação, concentre-se
em SDP e ICE. Ao acompanhar mídia, concentre-se em trilha, RTP, SSRC e payload
type. Ao estudar segurança, diferencie WSS da proteção de mídia WebRTC.

## Candidatos não são conexões prontas

Um candidato é uma possibilidade de endereço/porta para comunicação. Um
candidato `host` vem de uma interface local; `srflx` registra um endereço
observado por STUN; `relay` vem de uma alocação TURN. ICE também pode descobrir
endereços durante suas verificações, chamados `prflx`.

ICE compara candidatos locais e remotos e testa pares compatíveis. Um endereço
coletado pode estar bloqueado por firewall ou não ter rota a partir do outro
participante. Por isso, a lista no SDP não diz qual caminho será usado. A
seleção é uma etapa posterior. Essa distinção é parte do funcionamento de
[ICE descrito na RFC 8445](https://www.rfc-editor.org/rfc/rfc8445.html).

Na POC, `somente_retransmissao=false` usa política `All`: permite os caminhos
disponíveis. Isso não promete conexão direta. Com `true`, usa `Relay`: o ensaio
exige retransmissão. O texto `caminho_ice` é produzido a partir do par realmente
selecionado, e não a partir da presença de uma URL TURN no JSON.

## O que a biblioteca faz e o que sobra para o projeto

| Responsabilidade | Implementação nesta POC |
| --- | --- |
| Negociar descrições e estabelecer transportes WebRTC | webrtc-rs, acionado por `chamada.rs` |
| Escolher como os participantes trocam SDP | Protocolo próprio de salas, fora do webrtc-rs |
| Capturar câmera e microfone | FFmpeg/DirectShow, iniciado por Rust |
| Codificar VP8/Opus e produzir RTP local | FFmpeg |
| Inserir RTP codificado em uma trilha WebRTC | `TrackLocalStaticRTP::write_rtp` |
| Entregar RTP remoto à aplicação | `TrackRemote::poll` |
| Decodificar e reproduzir | FFmpeg/FFplay, controlados por `Midia` |
| Mostrar imagens e mensagens | Flutter, via FRB |
| Definir prazos, cancelamento, diagnóstico e política de erro | Código da POC |

`TrackLocalStaticRTP` é adequado porque o encoder já produz pacotes RTP.
O caminho não entrega RGBA ao webrtc-rs nem pede à biblioteca que abra a câmera.
Na recepção, `OnRtpPacket` também entrega mídia ainda codificada. Mostrar o
vídeo depende de continuar o caminho até o decoder e a interface.

## Segurança em dois caminhos

WSS protege a troca de mensagens com o servidor e o token compartilhado.
Na implantação proposta, Caddy termina esse TLS e encaminha WebSocket para Rust
no mesmo Windows, em loopback. O token controla entrada nas salas; não cria
contas individuais ou autorização por usuário.

A mídia usa a proteção do transporte WebRTC. Mesmo quando existe TURN, o helper
não envia RTP cru diretamente ao outro computador. TURN retransmite o tráfego
da conexão; o servidor de sinalização não se transforma em servidor de mídia.
Ter WSS funcionando não faz o tráfego de mídia usar TCP ou TLS do Caddy.

## Para continuar

Acompanhe agora [uma chamada completa](negociacao-e-transporte.md). Depois abra
o [guia dos arquivos](arquivos.md) com o código ao lado. Finalize com as
[dificuldades e decisões](dificuldades-e-decisoes.md), que separam limitações
confirmadas da biblioteca de simplificações da aplicação e falhas de infraestrutura.
