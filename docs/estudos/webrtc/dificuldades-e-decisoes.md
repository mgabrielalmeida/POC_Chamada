# Dificuldades, causas e decisões da integração

[Módulo WebRTC](README.md) · [Código](arquivos.md) ·
[Evidências de execução](../../desenvolvimento/validacao.md)

Nem toda falha observada numa chamada é um defeito de webrtc-rs. A experiência
da POC envolve a versão da biblioteca, o modo como integramos seus tipos, os
helpers e a rede real. Separar essas origens evita corrigir a camada errada.

## 1. WebRTC não resolve sozinho captura e codecs

**Origem: responsabilidade da integração.** A API de transporte usada recebe
mídia codificada; não entrega automaticamente câmera, microfone e alto-falante
do Windows. Escolher a biblioteca não conclui o caminho audiovisual.

**Solução adotada:** FFmpeg captura DirectShow, produz VP8/Opus e RTP; Rust
encaminha os pacotes para `TrackLocalStaticRTP`. FFmpeg/FFplay decodificam o
RTP remoto. Isso permitiu estudar os protocolos sem implementar outra integração
nativa de dispositivos e codecs no mesmo incremento.

**Custo e limite:** processos adicionais, pacote maior, cópias de buffers,
portas locais e mais recursos para encerrar. Não há cancelamento de eco nem
controle próprio que adapte bitrate do FFmpeg ao congestionamento. O caminho
é útil para a POC, mas precisa de nova avaliação para um aplicativo maior.

## 2. O RTP do helper precisa corresponder à trilha

**Origem: dois contratos de mídia independentes.** O encoder produz RTP com
seus parâmetros, enquanto a PeerConnection anuncia os parâmetros de sua trilha.
O decoder local, por sua vez, conhece apenas seu pequeno arquivo SDP.

**Solução adotada:** `enviar_rtp` ajusta o SSRC ao anunciado na trilha;
`receber_rtp` ajusta o payload type ao decoder e remove extensões de cabeçalho
que ele não negociou. Os payloads continuam codificados em VP8/Opus.

**Limite:** não é uma camada genérica para qualquer codec ou SDP. Acrescentar
codec exige alinhar registro, encoder, decoder, payloads e testes. Os testes
IPv6 verificam transporte e SSRC compatível; o teste sintético acrescenta
decodificação real. Uma evidência não substitui a outra.

## 3. Callbacks não podem virar leitores permanentes de mídia

**Origem: integração com a execução assíncrona.** Um callback é a notificação
de um fato, como uma trilha que apareceu. Se ele permanecer lendo essa trilha,
a entrega de outros eventos pode ser prejudicada.

**Solução adotada:** `Observador` encaminha notificações para um canal e retorna.
O laço da chamada cria tarefas de recepção independentes, registradas no
`JoinSet`. A solução mantém a propriedade e o encerramento das tarefas na sessão.

Isso é uma decisão preventiva visível no código, não um relato de que esse
problema foi reproduzido em uma chamada humana específica.

## 4. Só abrir IPv4 impedia a coleta útil em IPv6

**Origem: configuração da POC.** O cliente usava apenas `0.0.0.0:0`.
Ter uma URL com hostname AAAA não cria, por si só, um socket IPv6 no transporte.

**Solução adotada:** abrir wildcards IPv4 e IPv6. A biblioteca resolve os
endereços e seleciona os compatíveis com a família do socket. Não foi criado
um resolver DNS próprio nem uma conversão de IP no Flutter.

**Evidência e limite:** testes locais verificam resolução, STUN IPv6 e RTP
bidirecional em `::1`. O loopback dos helpers pode continuar IPv4 porque é
independente dos sockets da PeerConnection. Nada disso libera firewall externo.
Veja a [auditoria IPv6](../../desenvolvimento/auditoria-ipv6.md).

## 5. TURN com IPv6 literal perde os colchetes

**Origem: limitação confirmada nos fontes de webrtc 0.21.1.** O parser da URL
separa host e porta. Depois, `peer_connection/transport/turn_relayer.rs` monta
o destino com `format!("{}:{}", url.host, url.port)`. Um literal IPv6 precisaria
voltar a receber colchetes para distinguir seus dois-pontos da porta.

**Solução adotada:** usar hostname com registro AAAA e rejeitar TURN literal
antecipadamente em `rede.rs`, com uma mensagem que explica a alternativa.
O bug não foi corrigido por um fork da biblioteca. Foi contornado na topologia
usada pela POC, preservando as versões fixadas.

**Limite:** a regra não vale para todas as URLs IPv6. STUN literal e WSS com
colchetes têm caminhos distintos de parsing; o código/testes os aceitam.
O certificado de WSS ainda precisa ser válido para o endpoint acessado.

## 6. Alcançar TURN em IPv6 não determina a família do relay

**Origem: limitação do pedido de alocação em rtc-turn 0.21.1.** Os caminhos
`allocate` e `handle_allocate_response`, em `src/client/mod.rs`, enviam os
atributos de transporte/autenticação, mas não `REQUESTED-ADDRESS-FAMILY`.
O endereço do servidor e a família do endereço que ele vai alocar são decisões
distintas. Um socket IPv6 até o servidor não demonstra que o relay será IPv6.

**Solução adotada:** o coturn dedicado usa
`allocation-default-address-family=ipv6`. O ajuste ocorre no servidor da POC;
não implementa esse atributo no cliente. O exemplo público não usa `external-ip`,
pois o desenho usa IPv6 global sem mapeamento NAT IPv4.

**Evidência e limite:** a auditoria inicialmente registrou inspeção sem alocação
real. O [diagnóstico posterior](../../desenvolvimento/diagnostico-primeira-chamada.md)
registra ensaios com relay IPv6 e mídia sintética no próprio S. A topologia
não promete acesso de redes exclusivamente IPv4 nem relay entre famílias.

## 7. A espera pela coleta podia inviabilizar candidatos já disponíveis

**Origem: política de negociação da POC em interação com os prazos ICE.**
O fluxo sem trickle esperava reunir candidatos antes de enviar SDP. Um servidor
ou interface sem resposta atrasava a negociação inteira. Na investigação,
uma versão com espera de 30 segundos também falhou na repetição por prazo
interno do ICE. A causa específica da coleta incompleta no cliente B não foi
identificada remotamente; o registro não deve ser lido como prova de uma causa única.

**Solução adotada:** esperar no máximo dez segundos e usar os candidatos do
snapshot disponível. Durante a espera, continuar publicando vídeo local e
verificando helpers. Em Relay, exigir candidato relay; sem candidatos, falhar
com mensagem útil. O prazo de conexão da aplicação começa depois da troca SDP.

**Limite:** candidatos tardios não são enviados, e os candidatos disponíveis
podem não funcionar na rede remota. Uma evolução possível seria trickle ICE,
com novas mensagens, tratamento da ordem de chegada e testes. Ela ainda não
foi implementada e não é requisito para entender o fluxo atual.

## 8. Os receptores FFmpeg encerravam antes de a negociação terminar

**Origem: integração dos tempos de mídia e rede.** Os decodificadores são abertos
antes da conexão para estarem prontos quando chegar RTP. O receptor SDP usado
na investigação esperava dez segundos; a negociação podia durar mais.
O erro Windows `0xffffffea` correspondeu a `-22` no caso reproduzido.

**Solução adotada:** `-listen_timeout 120` nos receptores, mantendo os limites e
o cancelamento em Rust. Cada helper passou a ter nome e histórico limitado
de stderr, preservado mesmo se o canal de quadros estiver cheio.

**Evidência e limite:** o receptor foi ensaiado sem RTP e depois com decodificação.
Isso reproduziu o sintoma, mas o código de saída isolado no PC A não prova
qual helper daquela execução falhou. Os detalhes estão no diagnóstico datado.

## 9. O coturn estava bloqueando peers IPv6 públicos

**Origem: configuração de infraestrutura, não webrtc-rs.** A regra
`denied-peer-ip=::` era interpretada pelo coturn como faixa curinga. Os logs
mostraram peers públicos rejeitados por essa faixa. Conseguir uma alocação
relay não bastava se o servidor depois recusasse o destino da retransmissão.

**Solução adotada:** retirar essa regra do modelo e da configuração investigada,
preservando autenticação, quotas e filtros específicos. Não liberar loopback
no serviço público para contornar o problema.

**Evidência e limite:** o registro posterior contém transporte por relay e
decodificação sintética nos dois sentidos. Foi ensaio em S; continuam necessários
os cenários físicos de A/B em redes diferentes. A interpretação das faixas foi
rastreada nos [fontes coturn 4.18.0](https://github.com/coturn/coturn/blob/4.18.0/src/client/ns_turn_ioaddr.c)
citados no diagnóstico original.

## 10. Sinalização e encerramento também produziram falhas

**Origem: ciclo de vida da aplicação.** Um reset TCP Windows `10054` podia
ocorrer quando o servidor soltava a conexão logo após avisar `saiu`.
Separadamente, o sink Rust mantinha o stream Dart aberto após o cancelamento.

**Soluções adotadas:** o servidor reúne as metades do WebSocket e espera o
fechamento por até um segundo. No cliente, `encerrar_observacao` solta o sink
antes de cancelar a assinatura Dart. Helpers são encerrados/aguardados e
associados a Job Object para proteger também o fechamento abrupto.

Esses casos mostram por que estudar WebRTC exige acompanhar a sessão inteira,
mas não autoriza atribuir todos os problemas à biblioteca de protocolos.
Veja [servidor](../servidor/arquivos.md) e [ponte](../ponte-flutter-rust/arquivos.md).

## Limites ainda presentes e possíveis próximos estudos

| Limite atual | O que seria necessário estudar antes de ampliar |
| --- | --- |
| STUN/TURN apenas por UDP na integração | Caminho TCP/TLS da versão escolhida, validação de URLs e ensaios em redes que bloqueiam UDP |
| Sem renegociação ou ICE restart | Novo ciclo de estados, ordenação de SDP/candidatos e recuperação testada |
| Encerramento imediato em `Disconnected` | Política de recuperação e prazo antes de concluir falha |
| Bitrates fixos do FFmpeg | Medição de rede e ligação explícita entre feedback e encoder |
| RGBA copiado por eventos | Medição de custo e alternativa de apresentação, preservando controle da mídia em Rust |
| Diagnóstico de transporte limitado | Contadores por trilha, detecção de ausência de mídia, perda e jitter |
| Uma sessão por processo na API | Propriedade de recursos por sessão e contrato de eventos sem estado global |

Esta tabela descreve possibilidades de estudo, não funcionalidades entregues.
Ao atualizar webrtc-rs, reavaliar os contornos da versão 0.21.1 contra os fontes
e testes da nova versão antes de removê-los. Um bug histórico não deve ser
tratado como propriedade permanente de todas as versões futuras.
