# Arquitetura: quem faz o quê na chamada

[Trilha de estudos](../README.md) · Próximo: [WebRTC](../webrtc/README.md)

A POC é uma chamada de áudio e vídeo entre duas pessoas, cada uma em um Windows.
Sua pergunta principal é: conseguimos integrar a pilha WebRTC de Rust a uma
interface Flutter, mantendo rede e mídia sob controle de Rust?

Para responder, o projeto concentra o trabalho técnico em uma biblioteca Rust.
Flutter mostra controles e imagens. O servidor ajuda os dois clientes a trocar
as informações necessárias para começar a chamada. Depois, a mídia segue pelo
transporte WebRTC, diretamente ou com retransmissão TURN.

## Três caminhos diferentes

**Comandos e apresentação, dentro do cliente.** O clique em Criar / entrar chama
a API Dart gerada pelo FRB. A ponte entra na DLL Rust. Rust inicia a sessão e
devolve eventos de estado e quadros RGBA. A interface converte os pixels em
imagens visíveis; áudio não atravessa Dart.

**Sinalização, entre clientes e servidor.** Os dois clientes abrem WebSockets,
informam a sala e recebem os papéis de ofertante e respondente. Uma descrição
SDP apresenta codecs, parâmetros de transporte e candidatos de rede. O servidor
encaminha essa descrição ao outro cliente. Não decodifica vídeo nem escolhe o
par ICE.

**Mídia, entre as PeerConnections.** FFmpeg captura e codifica localmente.
Rust recebe RTP em loopback e o entrega à trilha do webrtc-rs. A biblioteca
negocia os protocolos e protege o tráfego entre clientes. Na recepção, Rust
entrega o RTP aos helpers locais para decodificação e reprodução.

Uma sala acessível demonstra parte do segundo caminho. Uma prévia de câmera
demonstra parte do primeiro e da captura. Nenhum dos dois, isoladamente,
demonstra o terceiro caminho completo.

## Responsabilidades e limites

| Parte | Faz | Entrega para a próxima parte |
| --- | --- | --- |
| Flutter | Formulário, botões, estado, vídeo local/remoto | Comandos pela ponte |
| FRB | Converte chamadas e eventos entre Dart e Rust | Parâmetros, resultados e stream |
| API Rust | Mantém uma sessão e um observador por processo | Tarefa da chamada e cancelamento |
| Chamada Rust | Coordena sinalização, PeerConnection, mídia e limpeza | RTP nas trilhas; eventos à API |
| webrtc-rs | Implementa negociação e transporte WebRTC | Estados, trilhas e pacotes recebidos |
| FFmpeg/FFplay | Captura, codifica, decodifica e reproduz | RTP, pixels RGBA e som |
| Servidor Rust | Mantém salas de duas pessoas em memória | Papéis e descrições SDP |
| Caddy | Recebe a sinalização WSS pública | WebSocket para Rust em loopback |
| coturn | Disponibiliza endereços relay e retransmite tráfego | Um possível caminho ICE para a mídia |

Rust é o responsável pela mídia, mesmo quando delega captura e codecs a processos
auxiliares. Isso significa que Rust escolhe argumentos, abre os helpers,
acompanha sua saúde e os encerra. Não significa que o código Rust implemente
um encoder VP8 ou Opus.

## Uma sessão do início ao fim

1. Flutter inicializa a DLL, verifica a ponte e assina eventos.
2. A API Rust impede uma segunda sessão simultânea e cria a tarefa da chamada.
3. A chamada carrega a configuração, abre sinalização e aguarda o outro participante.
4. Depois de `pronto`, prepara helpers, codecs e trilhas, e cria a PeerConnection.
5. Oferta e resposta carregam um retrato dos candidatos ICE disponíveis.
6. O estado `Connected` libera as tarefas que enviam RTP para as trilhas locais.
7. Trilhas remotas entregam RTP aos decodificadores; vídeo retorna pelo stream.
8. Saída, erro ou cancelamento passam pelo mesmo encerramento de recursos.

A chegada de `Connected` representa o transporte estabelecido. A mensagem da
tela diz que a transmissão começou, mas qualidade audiovisual precisa ser
observada e testada separadamente.

## Por que a POC foi mantida pequena

Há uma janela, uma sessão, duas pessoas por sala e uma negociação por conexão.
O vídeo é 320×240 a 10 fps; o áudio é Opus a 48 kHz em dois canais. A primeira
câmera e o primeiro microfone DirectShow são usados. Não há seleção de
dispositivos, contas, banco, histórico, reconexão automática ou chamadas em grupo.

Também não há cancelamento de eco, controle adaptativo próprio do bitrate do
FFmpeg nem sincronização audiovisual avançada. Essas escolhas tornam o caminho
WebRTC observável com pouco código, mas delimitam o que se pode concluir sobre
um aplicativo maior. Usar fones evita que o áudio reproduzido volte ao microfone
durante os ensaios; não implementa cancelamento de eco.

Não são usados Flutter WebRTC, MatrixRTC ou LiveKit. Não há um pacote próprio
compartilhado de protocolo: cliente e servidor usam JSON pequeno e explícito.
Os identificadores e comentários autorais priorizam português, preservando os
nomes exigidos pelas APIs e pelas ferramentas geradoras.

## Arquivo a arquivo: a raiz do projeto

| Arquivo | Como participa do funcionamento |
| --- | --- |
| [Cargo.toml](../../../Cargo.toml) | Declara `rust` e `servidor` como membros do workspace e o resolver 3. Os dois compartilham a resolução de dependências. |
| [Cargo.lock](../../../Cargo.lock) | Registra as versões efetivamente resolvidas, inclusive as dependências transitivas dos protocolos. É a referência para repetir um build com `--locked`. |
| [.gitignore](../../../.gitignore) | Exclui builds, cache, `.local`, logs e arquivos de ambiente. Evita misturar artefatos da execução com fontes. |
| [.gitattributes](../../../.gitattributes) | Define tratamento de texto e finais de linha LF para fontes/documentos e preserva imagens como binários. |
| [README.md](../../../README.md) | Apresenta o projeto e o início rápido. Encaminha para documentação operacional e estudos. |

Os módulos seguintes detalham os arquivos autorais das pastas `rust/`,
`servidor/`, `app/`, `config/` e `scripts/`, além da função dos arquivos gerados.

## Duas topologias, a mesma aplicação

Na LAN, um dos computadores pode hospedar a sinalização e o cliente não precisa
de servidores ICE externos. Na Etapa 2, um Windows S separado hospeda Rust,
Caddy e coturn. O desenho escolhido diante do CGNAT IPv4 usa IPv6 global, DNS
AAAA e clientes A/B com IPv6 funcional.

Os helpers continuam usando `127.0.0.1` mesmo quando a chamada usa IPv6: seu
tráfego é interno ao computador. Os sockets da PeerConnection são independentes
e abrem IPv4 e IPv6. A [infraestrutura](../infraestrutura/README.md) explica essa
fronteira; o [roteiro operacional](../../desenvolvimento/roteiro-entre-redes.md)
mostra como preparar cada máquina.

## Confira se entendeu

Ao ver vídeo local sem vídeo remoto, você consegue indicar quais partes já
funcionaram e quais ainda faltam investigar? Ao ver uma sala pronta, consegue
explicar por que UDP ainda pode estar bloqueado? Use os três caminhos do início
deste módulo para formular hipóteses antes de alterar configurações.
