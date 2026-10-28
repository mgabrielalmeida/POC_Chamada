# Estudos das tecnologias e do código

[Documentação](../README.md) · [Execução e testes](../desenvolvimento/README.md)

O objetivo principal é entender **como Rust usa webrtc-rs para estabelecer uma
chamada WebRTC**. Flutter, a ponte, a sinalização, os codecs e a infraestrutura
são estudados como partes desse caminho. Não é preciso ler arquivos gerados
antes de compreender os arquivos manuais.

Cada módulo apresenta o funcionamento geral e uma leitura arquivo a arquivo.
Os caminhos do código são links para os arquivos reais do repositório. A versão
estudada usa `webrtc` e `rtc` 0.21.1, FRB 2.13.0 e os helpers FFmpeg 9.0.2.
O estudo foi conferido com o checkout de 27/10/2026; resultados de execução estão
no [histórico de validação](../desenvolvimento/validacao.md).

## Percurso sugerido

| Ordem | Módulo | O que você deve conseguir explicar ao terminar |
| --- | --- | --- |
| 1 | [Arquitetura](arquitetura/README.md) | Quem controla a chamada e por onde passam negociação, mídia e imagens |
| 2 | [WebRTC e webrtc-rs](webrtc/README.md) | Como a PeerConnection negocia, encontra um caminho e transporta RTP |
| 3 | [Mídia](midia/README.md) | Como câmera/microfone viram RTP e como o RTP recebido vira imagem e som |
| 4 | [Servidor](servidor/README.md) | Como dois clientes se encontram e recebem papéis de oferta e resposta |
| 5 | [Ponte Flutter–Rust](ponte-flutter-rust/README.md) | Como comandos e eventos atravessam Dart e a DLL sem perder o controle da sessão |
| 6 | [Flutter](flutter/README.md) | Como a interface mostra eventos e evita apresentar quadros de uma chamada encerrada |
| 7 | [Infraestrutura](infraestrutura/README.md) | Como configuração, DNS, WSS, TURN e empacotamento sustentam o caminho completo |

Para acompanhar uma ação da tela, outra ordem útil é Flutter → ponte → WebRTC →
mídia. Para investigar ausência de mídia, comece por WebRTC e depois separe
captura, codecs, transporte e apresentação.

## O centro do estudo: WebRTC

O módulo principal foi dividido para permitir leitura progressiva:

1. [Conceitos e papel da biblioteca](webrtc/README.md): SDP, ICE, STUN, TURN,
   DTLS, SRTP, RTP, trilhas e a diferença entre `webrtc` e `rtc`.
2. [Uma chamada, passo a passo](webrtc/negociacao-e-transporte.md): da entrada na
   sala à coleta de candidatos, envio de pacotes e encerramento.
3. [Arquivo a arquivo](webrtc/arquivos.md): funções, estruturas, contratos e
   testes em `chamada.rs`, `rede.rs`, `lib.rs` e no manifesto Rust.
4. [Dificuldades e decisões](webrtc/dificuldades-e-decisoes.md): o que veio da
   biblioteca, o que veio da integração e o que ainda é uma limitação da POC.

## Como ler sem se perder

Comece pelo problema resolvido pelo módulo. Depois acompanhe uma entrada até sua
saída: por exemplo, uma URL de sinalização entra em `iniciar_chamada`, é validada
por `Rede` e só então chega a `connect_async`. Não tente memorizar todos os tipos
Rust antes de reconhecer esse fluxo.

Leia uma função junto com quem a chama e quem recebe seu resultado. Um `Ok(())`
de `iniciar_chamada` significa que a tarefa foi iniciada; a conexão efetiva é
informada depois pelo stream. Essa diferença explica vários comportamentos da tela.

Ao estudar uma correção, faça três perguntas: qual era o sintoma, em qual camada
estava a causa e qual evidência demonstra que a solução resolveu aquele caso?
Os registros de desenvolvimento ajudam a não confundir uma hipótese com uma
causa confirmada.

## Arquivos gerados, auxiliares e ausentes do Git

Os bindings FRB são explicados no módulo da ponte, e o executor C++ e os arquivos
Flutter gerados estão no módulo Flutter. Os testes são explicados no módulo do
código que exercitam; os comandos ficam no [guia de testes](../desenvolvimento/testes.md).

`target/`, `app/build/`, `.dart_tool/`, `ephemeral/` e `.local/` são saídas ou
dependências locais. Não são módulos autorais. Lançadores e ZIPs citados no
histórico foram produzidos fora do código versionado; sua existência naquela
rodada não significa que estejam presentes neste checkout.
