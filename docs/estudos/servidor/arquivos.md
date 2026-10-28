# Servidor: leitura arquivo a arquivo

[Funcionamento geral](README.md) · [Testes e comandos](../../desenvolvimento/testes.md)

## servidor/Cargo.toml

O [manifesto](../../../servidor/Cargo.toml) declara o pacote `sinalizacao`,
edição 2024. Tokio fornece runtime, TCP, tarefas e canais;
`tokio-tungstenite` 0.29.0 fornece WebSocket. `serde_json` trabalha com as
mensagens, `futures-util` com os streams e `anyhow` com erros/contexto.

Não há dependência de `webrtc`, FFmpeg ou Flutter. Essa ausência expressa uma
fronteira importante: o servidor apenas transporta a negociação.

## servidor/src/main.rs

[main.rs](../../../servidor/src/main.rs) é o executável. Lê
`SINALIZACAO_TOKEN`, rejeita valor vazio ou Unicode inválido e escolhe o endereço.
O primeiro argumento da linha de comando tem prioridade. Sem argumento, usa
`127.0.0.1:8080` com token ou `0.0.0.0:8080` sem token.

Abre `TcpListener`, mostra o endereço efetivamente associado e chama
`executar_com_token`. Um `tokio::select!` espera o servidor ou Ctrl+C.
Esse arquivo não conhece participantes ou conteúdo SDP; mantém a política de
inicialização separada da lógica testável da biblioteca.

## servidor/src/lib.rs

[lib.rs](../../../servidor/src/lib.rs) contém a lógica compartilhada pelo
executável e pelos testes.

| Elemento | Como funciona |
| --- | --- |
| `Envio` | Apelido para o canal que entrega mensagens à tarefa dona do WebSocket |
| `Salas` | Mapa compartilhado de nomes para vetores de `(id, Envio)` |
| `texto` | Serializa um `serde_json::Value` como mensagem WebSocket textual |
| `executar` | Conveniência para o modo sem token, usada também pelo teste sintético |
| `executar_com_token` | Valida o token opcional, cria mapa e aceita conexões, atribuindo identificadores |
| `atender` | Executa entrada, negociação, vigilância e limpeza de uma conexão |
| `fechar_conexao` | Aguarda o fechamento WebSocket por até um segundo antes de soltar TCP |

Leia `atender` em quatro momentos. Primeiro, handshake e primeira mensagem,
com limites. Segundo, autenticação e validação do nome, antes de ocupar a sala.
Terceiro, inclusão do canal no mapa e envio de `aguardando` ou `pronto`.
Quarto, o laço que atende mensagens da rede, fila de saída e pings.

Depois da entrada, o WebSocket é dividido em leitura e escrita. O resultado
do laço fica guardado; a limpeza ocorre antes de retorná-lo. Isso é relevante
para `?`: um JSON inválido termina o laço com erro, mas não deixa a sala ocupada.

Na limpeza, o mapa é removido somente se ainda contiver o `id` dessa conexão.
Ao participante restante são enfileirados `saiu` e fechamento. As metades do
WebSocket são reunidas, e `fechar_conexao` espera o encerramento por até um
segundo. Esse cuidado foi acrescentado após resets Windows `10054` poderem
fazer a última mensagem se perder; o [histórico](../../desenvolvimento/validacao.md)
registra a repetição dos testes após a correção.

O token é comparação com uma chave compartilhada de entrada. Saber o nome da
sala não substitui o token quando ele está ativo. Da mesma forma, o token não
identifica individualmente cada pessoa: todos os clientes autorizados usam a
mesma chave nessa POC.

## servidor/tests/salas.rs

[salas.rs](../../../servidor/tests/salas.rs) abre listeners reais em portas
escolhidas pelo sistema. Os clientes do teste são WebSockets reais, não mocks
do mapa interno. `receber` trata pings e impõe prazo para a resposta esperada;
`entrar` envia a primeira mensagem de sala.

Os cinco testes cobrem:

1. Dois participantes, oferta/resposta encaminhadas, rejeição do terceiro,
   aviso de saída e reutilização do nome da sala.
2. Nome e mensagem inválidos sem deixar o outro participante preso.
3. Token ausente/incorreto sem ocupar sala, seguido de entrada autorizada.
4. O mesmo contrato de token e sala por WebSocket em IPv6 loopback.
5. Token configurado como vazio sem desativar a proteção silenciosamente.

O helper `verificar_token_e_sala` reaproveita a verificação de autorização nos
endereços IPv4 e IPv6. Esses testes não abrem uma PeerConnection nem validam
um certificado WSS; comprovam o contrato do servidor no ambiente local.
