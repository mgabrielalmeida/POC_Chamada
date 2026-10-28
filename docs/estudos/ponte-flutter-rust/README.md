# Ponte Flutter–Rust: comandos, eventos e sessão

[Trilha de estudos](../README.md) · [Arquivo a arquivo](arquivos.md)

Flutter executa código Dart; a camada técnica é uma biblioteca Rust compilada
como `chamada.dll`. Flutter-Rust Bridge, ou FRB, gera o código que permite chamar
essa biblioteca e transportar valores entre as linguagens.

A ponte não é um servidor de rede. Ela atravessa uma fronteira entre runtimes
no mesmo processo. A rede começa depois, quando a sessão Rust abre a sinalização
e a PeerConnection. A versão fixada do pacote Dart, crate Rust e gerador é 2.13.0.

## Dois sentidos de comunicação

**Comandos:** Flutter chama verificar, iniciar ou encerrar. Alguns resultados
chegam por `Future`. **Eventos:** Rust publica estados e quadros num stream que
Flutter assina uma vez. Separar esses sentidos evita manter a chamada de botão
pendurada durante toda a conversa.

| API Rust | API Dart gerada | Contrato |
| --- | --- | --- |
| `verificar_ponte` | `verificarPonte` | Retorna uma mensagem fixa para confirmar acesso à DLL |
| `observar_chamada` | `observarChamada` | Registra o sink e produz stream de `EventoChamada` |
| `iniciar_chamada` | `iniciarChamada` | Cria a tarefa; não espera a conexão audiovisual ficar pronta |
| `encerrar_chamada` | `encerrarChamada` | Solicita parada e espera a tarefa liberar recursos |
| `encerrar_observacao` | `encerrarObservacao` | Solta o sink Rust que mantém o stream aberto |

Um `Future<void>` de iniciar concluído não significa que A já consegue falar
com B. Pode significar apenas que a tarefa foi criada e está aguardando a outra
pessoa. Erros posteriores chegam no stream. Já encerrar é uma espera pela
limpeza, pois a próxima chamada não deve disputar os dispositivos da anterior.

## O contrato dos eventos

`EventoChamada` contém `tipo`, `texto` e `rgba`. Eventos textuais têm vetor de
pixels vazio. Nos eventos de vídeo, `rgba` contém 307.200 bytes para 320×240
em RGBA. O modelo foi mantido pequeno para atravessar a ponte facilmente.

| tipo | Origem/uso |
| --- | --- |
| `pronta` | Publicado ao registrar observação; habilita a entrada na interface |
| `estado` | Progresso de espera, preparação, coleta ou conexão |
| `caminho_ice` | Par selecionado e indicação de caminho direto/TURN |
| `video_local`, `video_remoto` | Pixels para cada painel |
| `aviso` | Diagnóstico de helper; não é necessariamente término da chamada |
| `erro` | Falha da sessão, publicada durante o encerramento |
| `encerrada` | Limpeza concluída; interface pode liberar imagens e permitir nova entrada |
| `audio_teste` | Contagem de PCM no teste sintético; não é reprodução de áudio em Dart |

Usar strings simplifica a inspeção, mas permite divergências por erro de escrita.
Se adicionar evento, revise emissor Rust, binding quando o formato mudar,
consumidor Dart e testes. Mudar apenas o texto de estado não exige regenerar
o contrato; mudar a estrutura pública pode exigir.

## Por que há estado global

A POC possui uma janela, um observador e uma sessão por processo. `OBSERVADOR`
guarda o sink; `SESSAO` guarda o cancelamento e o handle da tarefa. É uma escolha
de escopo, não uma exigência do FRB ou do WebRTC.

O mutex da sessão é assíncrono porque iniciar/encerrar podem precisar esperar
a tarefa anterior. O mutex do observador é usado em operações curtas sobre o
sink. Ele não fica guardado enquanto a chamada aguarda rede ou dispositivo.

Se o observador desaparece ou rejeita um evento, o encaminhador solicita
cancelamento. Isso evita deixar uma chamada sem interface consumindo recursos.
Para encerrar o cliente normalmente, a ordem é: encerrar chamada, soltar
observação Rust e só então cancelar a assinatura Dart.

## Confira se entendeu

Por que `verificarPonte` funcionar não comprova câmera ou WebRTC? Por que cancelar
apenas a assinatura Dart não é suficiente para liberar toda a sessão? Abra
o [guia dos arquivos](arquivos.md) e acompanhe a propriedade do sink e da tarefa.
