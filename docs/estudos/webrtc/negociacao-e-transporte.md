# Da sala ao RTP: uma chamada passo a passo

[Módulo WebRTC](README.md) · [Arquivos](arquivos.md) · [Dificuldades](dificuldades-e-decisoes.md)

Leia este texto com [chamada.rs](../../../rust/src/chamada.rs) aberto. Considere
A como a primeira pessoa na sala e B como a segunda. Os dois executam o mesmo
código; o campo `ofertante` recebido do servidor muda o papel de cada um.

## 1. Validar antes de abrir dispositivos

`executar` cria um `Recursos` inicialmente vazio e disputa duas possibilidades
com `tokio::select!`: receber cancelamento ou executar `conectar`. Assim,
cancelar também funciona enquanto o cliente aguarda a outra pessoa.

`conectar` carrega `Rede`, valida a URL de sinalização e constrói a configuração
ICE. Um JSON inválido ou TURN mal configurado falha antes de ligar a câmera.
Depois abre o WebSocket com prazo de dez segundos e envia `entrar`, sala e token.

A primeira pessoa recebe `aguardando`. Nesse período, o cliente responde a
pings, mas ainda não inicia mídia. Quando B entra, os dois recebem `pronto`.
A recebe `ofertante=true`; B recebe `false`. Essa regra evita duas ofertas
simultâneas sem precisar implementar resolução de colisão de ofertas.

## 2. Preparar a mídia e declarar codecs

`Midia::iniciar` cria receptores e capturadores locais. O código guarda os
sockets em que chegará o RTP produzido pelos encoders e os endereços para os
quais enviará o RTP remoto. Os detalhes estão no [módulo de mídia](../midia/README.md).

Em seguida, `MediaEngine` recebe dois registros:

| Trilha | MIME | Relógio RTP | Canais | Payload type local |
| --- | --- | --- | --- | --- |
| Vídeo | `video/VP8` | 90.000 Hz | Não se aplica; campo 0 | 96 |
| Áudio | `audio/opus` | 48.000 Hz | 2 | 111 |

O relógio RTP não é a taxa de quadros do vídeo. O vídeo tem dez quadros por
segundo, mas seus timestamps usam uma escala de 90.000 unidades por segundo.
O payload type é um identificador no contexto da sessão; 96 não significa VP8
em qualquer RTP existente. Aqui seu significado vem do registro de codec e SDP.

Para cada mídia, o código gera um SSRC e cria `MediaStreamTrack` com os parâmetros
de codificação. A trilha `TrackLocalStaticRTP` envolve essa descrição. Ela receberá
os pacotes já codificados; ainda não os transmite nesse ponto.

## 3. Construir a PeerConnection

O builder recebe configuração ICE, `MediaEngine`, interceptores padrão e o
`Observador`. Recebe também `with_udp_addrs(vec!["0.0.0.0:0", "[::]:0"])`.
O zero da porta deixa a escolha ao sistema. Os wildcards permitem que o driver
prepare sockets para as interfaces elegíveis de cada família.

É diferente de abrir somente `127.0.0.1` ou `::1`, que limita o transporte ao
próprio computador. Também é diferente de anunciar literalmente `0.0.0.0` como
endereço remoto: a biblioteca expande a configuração para interfaces utilizáveis.

O resultado fica em `Arc<dyn PeerConnection>`, compartilhado entre a coordenação
e as operações da sessão. `add_track` adiciona as duas trilhas antes da oferta.
Registrar interceptores não cria automaticamente um controlador que ajuste
os argumentos do FFmpeg: essa ligação não foi implementada.

## 4. Trocar oferta e resposta

O fluxo é o seguinte:

1. A chama `create_offer(None)` e `set_local_description`.
2. A aguarda a coleta ICE, lê `local_description()` e envia `descricao` pelo servidor.
3. B converte o JSON em `RTCSessionDescription` e chama `set_remote_description`.
4. B chama `create_answer(None)` e `set_local_description`.
5. B aguarda sua coleta, lê sua descrição local e envia a resposta.
6. A aplica a resposta com `set_remote_description`.

O objeto serializado tem o tipo de descrição e o texto SDP. O servidor o
encaminha; a interpretação WebRTC pertence aos clientes. O código marca
`negociada=true` após aplicar a descrição remota e, no respondente, enviar a
resposta. Uma segunda mensagem de negociação causa erro: renegociação e ICE
restart não estão implementados no fluxo da POC.

## 5. Entender a coleta parcial

`Observador::on_ice_gathering_state_change` marca um canal `watch` quando a
biblioteca chega a `Complete`. `aguardar_coleta` espera esse sinal por até dez
segundos. Enquanto espera, publica quadros locais e verifica a saúde dos helpers.

Quando a coleta termina ou o prazo acaba, a função lê a descrição local naquele
momento. Conta linhas `a=candidate:` e quantas possuem `typ relay`. Sem candidato,
falha. No modo forçado, também falha se não houver relay.

Se o tempo acabou e há candidatos aceitáveis, envia esse **snapshot**: um retrato
dos candidatos disponíveis. Isso evita que uma interface ou servidor sem resposta
inutilize candidatos úteis já obtidos. Porém candidatos obtidos depois não são
enviados. A POC não implementa *trickle ICE*, isto é, o envio incremental de
candidatos em mensagens posteriores à descrição inicial.

Consequentemente, dez segundos não são uma garantia de coleta completa nem um
valor universal recomendado para todo aplicativo. É um compromisso desta POC,
adotado após falhas registradas no [diagnóstico](../../desenvolvimento/diagnostico-primeira-chamada.md).

## 6. Observar estado e trilhas sem prender callbacks

O `Observador` envia mudanças de estado e trilhas para um canal interno. O
callback `on_track` retorna depois disso. A leitura longa da trilha ocorre
em uma tarefa separada, guardada no `JoinSet` da sessão.

O laço principal usa `tokio::select!` para atender WebSocket, mudanças WebRTC,
eventos de mídia e um pulso de vigilância. Ele não precisa escolher entre ficar
esperando vídeo e continuar respondendo ao servidor.

Na primeira mudança para `Connected`, cria uma tarefa `enviar_rtp` por trilha.
`Failed`, `Closed` ou `Disconnected` terminam a sessão como erro. Encerrar em
`Disconnected` é a política simples adotada aqui; a aplicação não tenta recuperar
uma interrupção transitória nem implementa reconexão automática.

## 7. Enviar mídia

O FFmpeg produz RTP no loopback. `enviar_rtp` lê o datagrama, interpreta-o com
`Packet::unmarshal`, aceita os payload types 96 ou 111 e substitui o SSRC pelo
valor declarado para a trilha. Depois chama `write_rtp`.

O ajuste existe porque o encoder auxiliar e a PeerConnection não negociaram
diretamente entre si. Se cada um usar uma identificação diferente para a mesma
fonte, o pacote pode não corresponder ao que a sessão anunciou. Sequência,
timestamp e payload continuam vindo do encoder.

Esse filtro é específico do contrato de dois codecs desta POC. Não deve ser
copiado como um validador RTP/RTCP genérico. Os helpers usam portas RTP/RTCP
separadas e os tipos esperados estão fixados nos dois lados da integração.

## 8. Receber mídia

Quando chega uma trilha remota, `receber_rtp` chama `poll()` em um laço. Ao
receber `OnRtpPacket`, troca o payload type para o número esperado no SDP do
decoder local: 96 para vídeo ou 111 para áudio. Também remove as extensões do
cabeçalho WebRTC, que o decoder local não negociou.

O pacote é serializado com `marshal` e enviado por UDP a `127.0.0.1`, no endereço
do helper receptor. O helper de vídeo decodifica VP8 em RGBA; o de áudio
decodifica Opus e reproduz. No modo sintético, o áudio vira blocos PCM para o teste.

Há, portanto, dois contratos SDP: o da sessão WebRTC entre os clientes e os
pequenos arquivos SDP locais dos decodificadores. Os arquivos locais não
contêm candidatos ICE nem substituem a negociação entre computadores.

## 9. Verificar o caminho usado e os prazos

A cada segundo, a vigilância verifica os helpers. Quando conectado, busca o
primeiro sender, seu transporte e o par selecionado via
`ice_transport().get_selected_candidate_pair()`. Se um dos candidatos é relay,
o texto informa retransmissão TURN; se nenhum é, informa direto. Só publica
novamente quando o texto muda.

O diagnóstico descreve tipo, protocolo, família, endereço e porta de ambos os
candidatos. Ele consulta esse transporte; não é um relatório completo de perda,
jitter ou estatísticas independentes de cada trilha. Para o aceite forçado,
confira relay local/remoto nos dois clientes e mídia bidirecional.

| Prazo | Onde atua | O que não significa |
| --- | --- | --- |
| 10 s | Abertura do WebSocket | Não é o tempo máximo de espera por outra pessoa na sala |
| Até 10 s por coleta | Antes de enviar oferta/resposta | Não garante que todos os candidatos foram reunidos |
| 45 s | Espera pela descrição remota no laço de negociação | Não é a duração máxima de uma chamada conectada |
| 30 s | Transporte, contado após a troca de SDP | Não substitui os temporizadores internos do ICE |
| 120 s | Espera inicial dos receptores SDP auxiliares | Não impede Rust de encerrá-los antes em erro/cancelamento |

Os prazos do laço são observados no pulso de vigilância. Não devem ser
interpretados como garantias de execução com precisão de milissegundos.

## 10. Encerrar de forma previsível

Ao terminar `conectar` ou receber cancelamento, `executar` tenta fechar o
WebSocket por até um segundo, fecha a PeerConnection, aborta e aguarda as
tarefas RTP, e encerra/aguarda os helpers. Se houve erro, publica `erro`;
ao final publica `encerrada`.

Centralizar isso em `Recursos` evita repetir limpezas em cada ponto que pode
falhar. A API só conclui `encerrar_chamada` depois de aguardar a tarefa. O
Job Object Windows oferece uma proteção adicional para helpers em fechamento
abrupto; não substitui o encerramento normal.
