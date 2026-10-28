# Servidor: como duas pessoas se encontram

[Trilha de estudos](../README.md) · [Arquivo a arquivo](arquivos.md)

O servidor de sinalização resolve um problema anterior à mídia: A e B precisam
descobrir que entraram na mesma sala, decidir quem envia a oferta e trocar suas
descrições WebRTC. O servidor não precisa entender os pacotes VP8 ou Opus para
fazer isso.

Ele mantém um mapa de salas em memória. Cada sala contém no máximo dois
identificadores de conexão e seus canais de envio. Não há cadastro, banco ou
histórico. Reiniciar o processo elimina todas as salas.

## Protocolo da sala

| Mensagem | Sentido | Significado |
| --- | --- | --- |
| `entrar`, com `sala` e `token` | Cliente → servidor | Primeira mensagem, solicita entrada |
| `aguardando` | Servidor → primeiro cliente | Sala criada, falta outra pessoa |
| `pronto`, com `ofertante` | Servidor → ambos | Dois participantes; primeiro oferece, segundo responde |
| `descricao`, com objeto `descricao` | Cliente → servidor → outro cliente | Transporta a oferta ou resposta SDP |
| `saiu` | Servidor → participante restante | A sala terminou e será preciso entrar novamente |
| `erro`, com `mensagem` | Servidor → cliente | Rejeição, como token inválido ou sala cheia |

Por exemplo, a intenção inicial pode ser representada por:

```json
{"tipo":"entrar","sala":"teste","token":""}
```

Esse exemplo usa o modo LAN sem token. Em implantação autenticada, o valor
correto é fornecido pela configuração privada do cliente. O token só participa
da entrada: não é encaminhado ao outro cliente nem incluído nos logs pelo
código do servidor.

O nome da sala deve ter de um a 48 caracteres ASCII entre letras, números,
hífen e sublinhado. O servidor recebe JSON, mas não interpreta o SDP: valida
que a mensagem é `descricao` e que o campo correspondente é um objeto.
A PeerConnection remota faz a interpretação detalhada.

## Concorrência sem compartilhar o socket inteiro

Cada conexão é atendida por uma tarefa Tokio. O mapa de salas é compartilhado
com `Arc<Mutex<...>>`: `Arc` permite que várias tarefas possuam uma referência,
e o mutex protege alterações no mapa.

Uma conexão não escreve diretamente no socket da outra. Ela encontra seu canal
`mpsc`, clona o remetente e encaminha a mensagem. A tarefa dona do WebSocket
recebe pelo canal e envia ao cliente. Essa organização evita manter o mapa
travado enquanto espera uma operação de rede.

O canal de saída comporta 16 mensagens. `try_send` não espera indefinidamente
por um cliente lento. Na retransmissão de uma descrição, falha de envio retorna
erro e inicia a limpeza; notificações de pronto/saída usam envio de melhor
esforço. Portanto não se trata de um protocolo de entrega persistente.

## Vida e término da sala

Quando a segunda pessoa entra, a ordem do vetor define os papéis. Não há uma
eleição distribuída nem tentativa de resolver ofertas simultâneas nos clientes.
Essa regra basta para uma negociação inicial entre duas pessoas.

Se qualquer participante sai ou produz erro após entrar, o servidor remove a
sala inteira e avisa o outro. Quem ficou não aguarda automaticamente um substituto;
ambos precisam iniciar uma nova entrada.

A limpeza verifica se o identificador da conexão ainda pertence à sala.
Imagine uma tarefa antiga terminando atrasada depois que uma nova sala com o
mesmo nome já foi criada: sem essa verificação, ela poderia apagar a sala nova.
O nome sozinho não identifica a geração de uma chamada.

## Proteções e limites

O handshake e a primeira mensagem têm prazos de dez segundos. Frames e mensagens
WebSocket são limitados a 64 KiB. O laço envia pings a cada dez segundos e
encerra quando não observa mensagem recebida por mais de 35 segundos.
Qualquer mensagem atualiza o instante de atividade; não apenas `Pong`.

Na LAN, o servidor pode operar sem token. Com `SINALIZACAO_TOKEN`, verifica
a chave antes de consultar/criar a sala e escuta em loopback por padrão.
Caddy fornece WSS no desenho público. O backend Rust não termina TLS.

Esses cuidados atendem a POC. Não há autenticação individual, armazenamento,
controle de acesso por sala, distribuição entre servidores ou protocolo de
renegociação. A mídia também não depende de ser encaminhada pelo WebSocket,
mas a aplicação mantém esse canal durante toda a chamada e encerra se ele cair.

## Confira se entendeu

Explique por que a terceira pessoa é rejeitada sem derrubar as duas primeiras.
Depois siga a saída de A: que tarefa avisa B, quem remove a sala e por que a
checagem do identificador é importante? O [guia dos arquivos](arquivos.md)
mostra onde cada uma dessas decisões está implementada.
