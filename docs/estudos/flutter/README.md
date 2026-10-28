# Flutter: apresentar a chamada e encaminhar ações

[Trilha de estudos](../README.md) · [Arquivo a arquivo](arquivos.md)

A interface possui endereço do servidor, sala, botões de entrada/saída, estados
e dois painéis de vídeo. Ela não abre câmera, não negocia ICE e não reproduz
Opus. Seu papel é representar o estado que Rust comunica e encaminhar a
intenção do usuário pela ponte.

## Inicialização e entrada

`main()` inicializa o binding Flutter, carrega `RustLib` e chama `verificarPonte`.
Se a DLL não puder ser carregada, passa uma mensagem de erro para a tela.
Assim o app consegue explicar a falha sem fingir que está pronto para chamar.

Sem erro inicial, `initState` assina `observarChamada()`. O evento `pronta`
habilita a entrada. `_entrar` marca a interface como ativa, limpa mensagens e
imagens anteriores, e chama `iniciarChamada` com campos sem espaços nas pontas.

O estado visual ativo começa antes de a rede conectar. Isso desabilita os
campos durante a tentativa e permite cancelar a espera. Não é um indicador
de conexão WebRTC já estabelecida; o texto de estado traz o progresso real.

## Dos pixels a uma imagem

Os eventos de vídeo carregam RGBA já decodificado em Rust/FFmpeg.
`decodeImageFromPixels` cria uma `ui.Image` para apresentação. O nome dessa API
não significa que Dart esteja decodificando VP8 ou realizando WebRTC: o dado
recebido já é uma matriz de pixels.

Cada painel admite uma conversão em andamento. Se chegar outro quadro antes
de ela terminar, ele é descartado. Quando uma imagem nova substitui a anterior,
a antiga recebe `dispose()` para liberar o recurso gráfico.

`_geracao` protege contra callbacks atrasados. Imagine encerrar enquanto um
quadro está sendo convertido: o callback pode terminar depois da limpeza.
Ele compara a geração capturada com a atual e descarta a imagem se a sessão
mudou, se a tela desmontou ou se a chamada deixou de estar ativa.

A prévia “Seu vídeo” usa `Transform.flip(flipX: true)`. A transformação é visual,
depois da captura e codificação; o quadro transmitido e o painel remoto mantêm
a orientação original.

## Eventos e encerramento

`_receber` separa vídeo de texto. `estado` atualiza a mensagem; `caminho_ice`
fica em um campo próprio para continuar visível; `erro` e `aviso` aparecem como
diagnóstico. `encerrada` desativa a chamada e libera as imagens.

O botão Encerrar marca `_encerrando` e aguarda `encerrarChamada`. A saída normal
da janela usa `AppLifecycleListener` para aguardar a mesma liberação. Em
`dispose`, `_liberarCliente` encerra a chamada, solta o observador Rust e cancela
a assinatura. Como `dispose` não é assíncrono, esse trabalho é disparado sem
espera ali; a espera da saída normal e a proteção Windows dos helpers têm
papéis complementares.

## O que observar ao estudar a tela

Relacione cada campo visual a um evento ou ação. `_ativa` não é uma leitura
direta da PeerConnection. `_erro` também pode apresentar aviso de helper que
não terminou a chamada. A tela é deliberadamente pequena; a interpretação do
diagnóstico continua dependendo da camada que o emitiu.

Abra o [guia dos arquivos](arquivos.md) para separar a interface autoral do
executor C++ gerado pelo Flutter. Essa distinção ajuda a não procurar lógica
de RTP dentro do código de criação da janela.
