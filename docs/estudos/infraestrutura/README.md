# Infraestrutura: configuração, rede pública e pacote

[Trilha de estudos](../README.md) · [Arquivo a arquivo](arquivos.md)

Este módulo explica por que os serviços e configurações existem. Os comandos
para preparar cada computador ficam no [roteiro entre redes](../../desenvolvimento/roteiro-entre-redes.md).
Separar explicação de execução evita que uma leitura de estudo seja confundida
com uma implantação já realizada.

## A, B e S

A e B são participantes: cada um executa Flutter, a DLL Rust e os helpers.
S é o Windows que hospeda três processos distintos: sinalização Rust,
Caddy e coturn. Na LAN, o servidor de sinalização pode estar em um participante;
na topologia de internet documentada, ele fica em uma terceira máquina.

| Caminho | Serviço | Papel |
| --- | --- | --- |
| A/B → domínio WSS | Caddy, TCP 443 | Recebe sinalização com TLS público |
| Caddy → 127.0.0.1:8080 | Sinalização Rust | Autentica entrada, mantém salas e encaminha SDP |
| A/B → STUN/TURN UDP 3478 | coturn | Responde à descoberta/alocação |
| Tráfego retransmitido | Portas UDP 49160–49200 no coturn de referência | Transporta o caminho relay da mídia |
| Mídia direta A ↔ B | PeerConnections | Usa caminho ICE sem relay quando selecionado |

TCP 80 também faz parte do roteiro de HTTPS público/ACME. O backend 8080 não
precisa ficar exposto na implantação com Caddy. Abrir 443 e entrar na sala
não testa a faixa UDP nem as permissões da retransmissão.

## Por que o desenho usa IPv6

A investigação original encontrou IPv4 sob CGNAT. Nesse cenário, uma regra
no roteador doméstico não controla o NAT adicional da operadora. O desenho
adotado usa um IPv6 global de S e um registro DNS AAAA apontando para ele.
A/B também precisam de conectividade IPv6 utilizável.

IPv6 global não significa entrada automaticamente permitida. Firewalls do
Windows, roteador e política do provedor continuam importando. Prefixos podem
mudar; o DNS precisa acompanhar. O endereço não temporário escolhido também
precisa ser conferido novamente se a rede mudar.

O hostname registrado no histórico é `poc-chamada.dynv6.net`. Isso é contexto
da implantação investigada, não uma checagem atual de disponibilidade. Os
arquivos versionados mantêm `chamada.exemplo.org` como modelo a preencher.

## Quatro informações que não devem ser confundidas

**URL de sinalização:** informada no campo da tela; diz onde está o WebSocket.
**Token de sinalização:** lido do JSON; autoriza a entrada no servidor Rust.
**URLs ICE:** lidas do JSON; dizem onde descobrir endereço e solicitar relay.
**Usuário/senha TURN:** autenticam alocações no coturn, não a sala de sinalização.

Esses valores viajam por caminhos diferentes. Caddy não cria uma alocação TURN
porque a sinalização foi autenticada. O servidor Rust também não distribui
automaticamente credenciais temporárias de TURN nesta implementação.

## Configuração por processo

`CHAMADA_CONFIG` contém o caminho de um JSON local. Rust o lê a cada nova
entrada na sala, antes dos dispositivos. A variável precisa existir no ambiente
do processo que inicia o aplicativo. Editar o JSON antes da próxima chamada
pode ser percebido; definir uma variável em outro terminal não modifica o
ambiente de uma instância já aberta.

Sem variável, o cliente usa LAN sem servidores ICE externos. Com variável
apontando para arquivo inexistente/inválido, falha. Não há escolha silenciosa
de outra configuração. Campos desconhecidos também geram erro, ajudando a
identificar nomes digitados incorretamente.

## TURN IPv6 e UDP

O coturn de referência escuta e aloca no IPv6 global de S. Define a família
padrão de alocação como IPv6 para contornar a ausência do atributo correspondente
no cliente `rtc-turn` 0.21.1. O hostname AAAA também contorna a remontagem
incorreta de TURN literal IPv6. Esses dois problemas são diferentes e estão
explicados no [módulo WebRTC](../webrtc/dificuldades-e-decisoes.md).

A integração aceita UDP; não há suporte validado na POC a `turns:` ou TURN/TCP.
O fato de a sinalização usar WSS não oferece uma alternativa automática para
redes que bloqueiam UDP. Também não transforma o servidor somente IPv6 em
um serviço alcançável por clientes exclusivamente IPv4.

## Compilar não é instalar nem validar

O pacote cliente reúne executável Flutter, engine, dados, DLL Rust, helpers e
licenças. Copiar só `poc_chamada.exe` perde partes necessárias. O servidor Rust
é outro executável; Caddy e coturn não são dependências compiladas pelo workspace.

Gerar o pacote comprova a etapa de build. Abrir a interface comprova mais um
trecho. Negociar por WSS e selecionar relay comprovam outros trechos. O aceite
final exige mídia real, duração e repetição nas duas redes, conforme os
[critérios de desenvolvimento](../../desenvolvimento/etapas.md).
