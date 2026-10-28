# Mídia: leitura arquivo a arquivo

[Funcionamento geral](README.md) · [Adaptação RTP](../webrtc/negociacao-e-transporte.md)

## rust/src/midia.rs

[midia.rs](../../../rust/src/midia.rs) possui os processos, sockets locais,
arquivos temporários e tarefas de leitura. `LARGURA` e `ALTURA` definem
320×240; a interface e o teste de quadros precisam continuar compatíveis com
esse contrato se a resolução mudar.

### Estruturas e propriedade

`Midia` expõe os sockets `video`/`audio` que recebem a captura e os endereços
`destino_video`/`destino_audio` dos decodificadores. Mantém os sockets RTCP
reservados, os filhos, um `JoinSet`, o `TempDir` e, no Windows, `TrabalhoWindows`.

`ProcessoMidia` reúne um nome, o `Child` e o histórico de stderr. O histórico
usa `Arc<Mutex<VecDeque<String>>>` porque a tarefa que lê stderr escreve nele,
enquanto `verificar_processos` o lê para explicar uma saída inesperada.

`EventoMidia` é interno: `Quadro { local, rgba }`, `AudioDecodificado` e
`Erro(String)`. O módulo não conhece widgets Flutter nem converte RTP em
`EventoChamada`; essa tradução pertence a `chamada.rs`.

### Localização e abertura dos executáveis

`binario` procura primeiro ao lado do executável em execução. Se não encontrar,
procura na pasta `.local/ffmpeg-9.0.2-essentials_build/bin` relativa ao manifesto
Rust usado na compilação. Isso atende pacote distribuído e desenvolvimento.
Não depende de mudar o PATH global.

`comando` cria o processo com stdin fechado, stderr capturado, stdout fechado
por padrão e `kill_on_drop(true)`. O stdout é reaberto apenas onde será usado.
No Windows, `creation_flags(0x08000000)` evita criar janela de console para helpers.

`dispositivos` executa a listagem DirectShow e procura no stderr as primeiras
linhas que terminam em `(video)` e `(audio)`. A listagem pode retornar código
de erro apesar de conter dispositivos; por isso o código interpreta o texto.
Falhar em localizar câmera/microfone gera orientação sobre permissões.

### Portas RTP/RTCP

`portas_recepcao` tenta até cem pares. Escolhe uma porta par na faixa
20.000–39.998 e reserva a seguinte para RTCP. Reservar só RTP permitiria que a
porta vizinha fosse ocupada pelo RTP de outra trilha. A faixa evita disputar
as portas efêmeras padrão do Windows usadas nas saídas dos helpers.

Para captura, Rust mantém RTP e a reserva RTCP. Para os decodificadores, as
reservas são liberadas imediatamente antes de iniciar o processo receptor,
pois ele precisa abrir as portas. Existe um intervalo entre liberar e o
helper associar a porta; não se trata de uma transferência atômica de socket.
O diagnóstico de processos ajuda a identificar falhas dessa abertura.

### Midia::iniciar

Leia a função nesta ordem:

1. Reserva pares, converte os sockets de captura para Tokio e define destinos.
2. Cria `TempDir` e os dois SDP locais com VP8/90000 e opus/48000/2.
3. Cria o proprietário `Midia` e o Job Object.
4. Prepara receptor de vídeo, que lê SDP e escreve RGBA no stdout.
5. Prepara receptor de áudio: FFplay real ou FFmpeg/PCM sintético.
6. Enumera dispositivos reais, ou escolhe fontes `lavfi` sintéticas.
7. Inicia câmera com saída RGBA e VP8/RTP e microfone com Opus/RTP.

Os receptores usam whitelist de protocolos `file,udp,rtp`, endereço local
`127.0.0.1`, parâmetros de baixa latência e `-listen_timeout 120`. Esse prazo
evita a expiração inicial antes do fim da negociação; cancelamento e falha
de conexão continuam encerrando os processos por Rust.

O vídeo usa libvpx, modo realtime, `cpu-used=8`, sem atraso em frames e GOP 10.
Com 10 fps, o intervalo nominal entre keyframes é de cerca de um segundo.
O áudio usa libopus, aplicação `voip`, quadros de 20 ms e dois canais a 48 kHz.
As saídas RTP limitam o tamanho de pacote a 1200 bytes e usam portas de origem
escolhidas pelo sistema, separadas dos destinos locais reservados.

### adicionar e os leitores

`adicionar` inicia o filho, associa-o ao Job Object e configura as tarefas.
Quando há vídeo no stdout, lê exatamente 307.200 bytes antes de emitir um
quadro: uma leitura parcial do pipe não é, sozinha, uma imagem completa.
Clona o buffer para o evento e usa envio sem espera para evitar acumulação.

No teste de áudio, outro leitor recebe blocos de 3.840 bytes de PCM s16le.
São 960 amostras por canal × dois canais × dois bytes, ou 20 ms a 48 kHz.
O teste conta blocos decodificados; não envia esse PCM ao Flutter para tocar.

O leitor de stderr mantém oito linhas, truncadas em mil caracteres cada,
e tenta publicar avisos. O histórico persiste mesmo se esse envio falhar por
canal cheio. O log original do helper pode estar em inglês e é preservado.

### verificar_processos, encerrar e TrabalhoWindows

`verificar_processos` usa `try_wait`, sem bloquear esperando o helper terminar.
Se algum já saiu, retorna erro com nome, status e histórico. Mesmo uma saída
com código zero é inesperada durante uma sessão que deveria continuar ativa.

`encerrar` chama `kill` e `wait` para cada filho e depois aborta/aguarda tarefas.
`TrabalhoWindows::novo` cria um Job Object com `KILL_ON_JOB_CLOSE`;
`associar` registra cada processo; `Drop` fecha o handle. O `unsafe` está nessa
fronteira com a API C do Windows, não na lógica de negociação WebRTC.

## rust/examples/diagnosticar_midia.rs

O [exemplo](../../../rust/examples/diagnosticar_midia.rs) chama `Midia::iniciar`
com dispositivos reais. Aguarda um quadro local RGBA e um RTP Opus com payload
não vazio, por até 15 segundos, verificando os processos. Depois chama
`encerrar`, inclusive quando o ensaio retorna erro ou expira.

Ele isola captura/codecs de sinalização e PeerConnection. Não comprova que o
áudio foi reproduzido na outra máquina ou que o caminho da internet funciona.
Seu lugar na investigação é anterior ao ensaio audiovisual completo.

## scripts/preparar-midia.ps1

O [script](../../../scripts/preparar-midia.ps1) resolve a raiz a partir de
`PSScriptRoot`, cria `.local`, baixa o ZIP fixado se ausente, verifica SHA-256
e extrai se o executável esperado ainda não existir. Um hash diferente interrompe
a preparação. Não instala serviço nem altera PATH.

O hash valida o ZIP baixado; se os arquivos já extraídos forem modificados,
esse fluxo não faz uma auditoria individual de todos os executáveis extraídos.
O script e o fallback de `binario` devem continuar apontando para a mesma
versão. O empacotamento dessa dependência é explicado em
[infraestrutura](../infraestrutura/arquivos.md).
