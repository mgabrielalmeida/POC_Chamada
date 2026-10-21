# Guia didático do código

## Por onde começar

Leia o README para executar. Abra `app/lib/main.dart`, `rust/src/api.rs`,
`rust/src/chamada.rs`, `rust/src/midia.rs` e `servidor/src/lib.rs`, nessa ordem.
São os arquivos manuais centrais. O guia da Etapa 1 detalha comandos e dependências.

## Uma ação atravessando o projeto

1. A pessoa preenche servidor e sala e clica em Criar / entrar.
2. `_entrar()` em Dart chama `iniciarChamada`, um binding gerado pelo FRB.
3. `iniciar_chamada` em Rust cria uma sessão e um canal de cancelamento.
4. `chamada::executar` abre a sinalização. A primeira pessoa aguarda a segunda.
5. Quando a sala fica pronta, `Midia::iniciar` prepara captura e codecs.
6. `PeerConnectionBuilder` cria a conexão WebRTC com trilhas VP8 e Opus.
7. A primeira pessoa envia a oferta SDP; a segunda envia a resposta. Os endereços
   ICE são reunidos antes de enviar o SDP completo, sem mensagens ICE separadas.
8. Ao conectar, Rust encaminha o RTP dos encoders para as trilhas webrtc-rs.
9. Trilhas remotas entregam RTP aos decodificadores locais. Áudio é reproduzido
   pelo helper e vídeo RGBA retorna por Rust e FRB à interface.
10. Encerrar sinaliza cancelamento e aguarda a liberação de conexões e processos.

## Flutter apresenta

`main()` carrega a DLL e verifica a ponte. `TelaChamada` mantém campos visuais,
texto de estado e duas imagens. Ela assina `observarChamada()` e chama apenas
`iniciarChamada` e `encerrarChamada` em resposta aos controles.

`_apresentar()` converte os bytes RGBA em `ui.Image`; não decodifica VP8, não
reproduz áudio e não negocia WebRTC. Se uma imagem ainda está sendo preparada,
o próximo quadro é descartado. Um contador de geração impede que um callback
atrasado reapresente vídeo de uma chamada encerrada.

`app/windows/` contém o executor gerado pelo Flutter. A alteração própria de
compilação fica em `rust.cmake`: compila a biblioteca e copia a DLL e os helpers.
Nenhuma lógica de chamada foi colocada no executor C++.

## A API Rust controla uma sessão

`api.rs` expõe a API da ponte e `EventoChamada`, com tipo, texto e bytes RGBA.
Há uma sessão ativa e um assinante por processo. O mutex da sessão impede iniciar
outra chamada enquanto a anterior ainda está ativa ou encerrando.

`iniciar_chamada` retorna após iniciar o trabalho; o stream informa sucesso,
erros e quadros. `encerrar_chamada` espera a tarefa terminar, permitindo iniciar
novamente sem deixar o microfone ou a câmera ocupados.

Ao destruir o cliente, `encerrar_observacao` solta o sink mantido por Rust antes
de cancelar a assinatura Dart. Essa ordem fecha o stream e evita manter uma
porta Dart aberta depois da chamada.

## WebRTC e sinalização do cliente

`chamada.rs` cria a conexão, negocia SDP e encaminha mídia. `Observador` recebe
estados e trilhas do webrtc-rs; o callback só envia notificações internas, evitando
bloquear o driver de rede com leituras longas.

`enviar_rtp` ajusta o SSRC do pacote do encoder à trilha anunciada na negociação.
`receber_rtp` ajusta o payload type à configuração do decoder local. Os helpers
conhecem VP8 e Opus, mas não participam de ICE, DTLS ou SRTP entre os dispositivos.

`Recursos` guarda WebSocket, conexão WebRTC, mídia e tarefas. O bloco final de
`executar` libera tudo tanto na saída normal quanto no erro ou cancelamento.

`rede.rs` lê o JSON opcional indicado por `CHAMADA_CONFIG`, valida a sinalização
e monta a configuração ICE. Sem arquivo, mantém LAN sem servidores externos.
STUN descobre endereços; TURN pode retransmitir mídia. A opção de retransmissão
forçada exige TURN. A configuração é lida ao entrar, antes dos dispositivos,
sem alterar a API gerada ou colocar decisões de rede no Flutter.

Depois de conectar, `chamada.rs` consulta o transporte ICE alcançado pelo sender
de mídia. O evento `caminho_ice` informa se o par selecionado contém `relay`.
Flutter guarda esse texto separado do estado para apresentá-lo durante a chamada.
O guia da Etapa 2 explica configuração, credenciais e limites desse diagnóstico.

## Captura e codecs

`midia.rs` localiza os executáveis, enumera dispositivos DirectShow, cria os
helpers e lê suas saídas. Os processos ficam invisíveis. `Midia` possui os filhos,
as tarefas de leitura e os arquivos temporários, além dos sockets locais.

Os helpers só trocam RTP com o próprio Rust via loopback. Dois canais limitados
separam leitura de quadros e envio à interface; quadros podem ser descartados.
`verificar_processos` percebe quando algum helper encerra. `encerrar` mata e
aguarda os processos e cancela suas tarefas. O Job Object Windows cobre também
fechamentos abruptos em que destrutores Rust não executam.

## O servidor reúne as duas pessoas

`servidor/src/main.rs` escolhe endereço e inicia `sinalizacao::executar`.
`servidor/src/lib.rs` mantém salas em memória e encaminha as descrições SDP.
A primeira conexão oferece; a segunda responde. Ao sair alguém, a sala inteira
é removida e a outra pessoa é avisada. Um identificador impede que a limpeza de
uma conexão antiga apague uma nova sala com o mesmo nome.

`executar` mantém o modo LAN; `executar_com_token` protege a entrada.
`main.rs` lê `SINALIZACAO_TOKEN` e, quando configurado, escolhe loopback como
endereço padrão para trabalhar atrás do proxy WSS. `atender` confere o token
antes de criar a sala; encaminha apenas descrições, sem credenciais.

`fechar_conexao` aguarda por até um segundo o fechamento WebSocket antes de
liberar o TCP. Isso evita perder a última mensagem por reset no Windows, tanto
na rejeição de entrada quanto na saída da sala.

## Arquivos gerados e verificações

Não edite `rust/src/frb_generated.rs` ou `app/lib/src/rust/` manualmente. Altere
`api.rs` e execute o gerador em `app/`. Nomes e comentários das ferramentas podem
ficar em inglês; código próprio e explicações usam português.

`servidor/tests/salas.rs` verifica o protocolo com WebSockets reais.
`rust/tests/chamada_sintetica.rs` verifica transporte e decodificação com fontes
geradas e duas chamadas seguidas. `rust/examples/diagnosticar_midia.rs` verifica
captura real isoladamente. `app/tool/verificar_ponte.dart` verifica a travessia
Dart → DLL. Os limites de cada evidência estão em `04-validacao.md`.
