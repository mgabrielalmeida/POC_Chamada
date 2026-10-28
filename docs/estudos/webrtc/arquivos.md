# WebRTC: leitura arquivo a arquivo

[Módulo WebRTC](README.md) · [Fluxo completo](negociacao-e-transporte.md)

## rust/Cargo.toml: quais peças entram no cliente

O [manifesto](../../../rust/Cargo.toml) cria o crate `chamada` em dois formatos:
`cdylib`, a DLL consumida pela ponte, e `rlib`, usado pelo código Rust e testes.
A edição é 2024 e o pacote não é destinado à publicação.

`webrtc = "=0.21.1"` fixa a camada assíncrona. `rtc` também está fixado em 0.21.1,
com `default-features=false` nessa declaração, e fornece tipos de protocolo.
As features transitivas efetivas ainda dependem da resolução do Cargo; essa
linha isolada não descreve todas as features habilitadas por `webrtc`.

Tokio executa tarefas, canais, sockets e processos. `futures-util` fornece
operações de stream/sink. `async-trait` permite a implementação dos callbacks
assíncronos usada pelo observador. `bytes` e os traits de marshal ajudam a
converter datagramas em estruturas RTP e vice-versa. `serde`/`serde_json`
tratam configuração e sinalização; `anyhow` acrescenta contexto aos erros.

`tokio-tungstenite` está fixado em 0.29.0 com `rustls-tls-native-roots`, usado
na sinalização WSS. `tempfile`, `rand` e `windows-sys` atendem arquivos
temporários, identificadores/portas e Job Objects. FRB expõe a API ao Dart.
`sinalizacao` é uma dependência de desenvolvimento por caminho: permite que
os testes iniciem o servidor real, sem torná-lo parte do runtime do cliente.

## rust/src/lib.rs: a porta de entrada da biblioteca

[lib.rs](../../../rust/src/lib.rs) declara os módulos. `api`, `chamada` e
`midia` são públicos para atender ponte, testes e exemplo de captura.
`rede` e `frb_generated` ficam internos. Este arquivo não abre sockets nem
inicia uma chamada só por ser carregado.

A API da ponte tem seu próprio [módulo de estudo](../ponte-flutter-rust/README.md).
O uso de `pub` em módulos não transforma todas as funções em funções Dart:
o gerador está configurado para ler `crate::api`.

## rust/src/rede.rs: transformar configuração em política

[rede.rs](../../../rust/src/rede.rs) é lido no início de cada chamada.
`Rede` guarda token, servidores ICE e `somente_retransmissao`.
`ServidorIce` guarda URLs, usuário e senha. A estrutura externa aceita campos
omitidos por meio dos valores padrão; campos desconhecidos são rejeitados.

| Função | Entrada e saída | Decisão relevante |
| --- | --- | --- |
| `carregar` | Ambiente/arquivo → `Rede` | Sem `CHAMADA_CONFIG`, usa padrão LAN; arquivo definido mas inválido não vira LAN silenciosamente |
| `ler` | Texto JSON → estrutura | Substitui o erro detalhado do parser por mensagem que não reproduz valores secretos |
| `configuracao_ice` | Lista de servidores/política → `RTCConfiguration` | Valida as URLs e seleciona `All` ou `Relay` |
| `validar_sinalizacao` | URL → resultado de validação | Aceita WS/WSS e exige proteção adequada quando usa token fora do loopback |

`configuracao_ice` rejeita lista de URLs vazia em uma entrada, esquemas diferentes
de `stun:`/`turn:`, credenciais embutidas na URL e consultas de transporte que
não correspondam ao caminho UDP aceito. Depois chama `RTCIceServer::urls()`
para a validação da biblioteca. O erro original também é ocultado para evitar
reproduzir credenciais. A validação específica de TURN IPv6 literal é explicada
nas [dificuldades](dificuldades-e-decisoes.md).

No modo Relay, a ausência de uma entrada TURN causa erro antes da conexão.
Ter entrada TURN válida não comprova autenticação no servidor: isso só pode
ser observado durante a alocação e o transporte.

`validar_sinalizacao` usa `http::Uri`, não separação manual por dois-pontos.
Reconhece `localhost`, `127.0.0.1` e `[::1]` como exceções de loopback para WS
com token. Exige token não vazio em WSS e rejeita WS com token fora do loopback.
O modo LAN sem token continua aceitando WS; a regra não detecta automaticamente
se um IP sem token está na internet. A implantação correta continua necessária.

O bloco `testes` no mesmo arquivo contém seis testes: padrão LAN, política
Relay, entradas inválidas sem vazamento de segredo, regras WS/WSS, URLs IPv6 e
o diagnóstico para TURN literal. Eles verificam configuração, sem contatar TURN.

## rust/src/chamada.rs: coordenação da sessão

[chamada.rs](../../../rust/src/chamada.rs) conecta as peças. Comece pelas
estruturas para entender quem possui cada recurso:

| Estrutura | Papel |
| --- | --- |
| `Mudanca` | Notificação interna de estado WebRTC ou trilha remota |
| `Observador` | Implementa `PeerConnectionEventHandler`; sinaliza coleta e encaminha mudanças |
| `Recursos` | Guarda WebSocket, PeerConnection, `Midia` e tarefas RTP para a limpeza comum |
| `Evento::texto` | Constrói evento sem pixels para a ponte, usando `EventoChamada` |

`executar` é o contorno da sessão: aceita endereço, sala, escolha de mídia
sintética, canal de eventos e canal de parada. Delega o trabalho a `conectar`,
mas mantém a limpeza fora dessa função. Assim, `?` em uma operação interna não
pula o encerramento final de recursos que já foram registrados em `Recursos`.

`conectar` contém cinco blocos que vale ler separadamente: entrada na sala;
preparação de mídia/codecs; construção da conexão; criação da oferta quando
necessária; e laço de eventos. O [passo a passo](negociacao-e-transporte.md)
acompanha esses blocos na ordem de execução.

| Função | Trabalho específico | Cuidado ao modificar |
| --- | --- | --- |
| `aguardar_coleta` | Espera conclusão ou dez segundos, verifica helpers e lê o SDP disponível | Não tratar o snapshot como coleta necessariamente completa; manter exigência relay no modo forçado |
| `publicar_midia` | Converte eventos internos em vídeo local/remoto, áudio de teste ou aviso | Usa `try_send`; eventos desse caminho podem ser descartados quando o canal enche |
| `descrever_candidato` | Formata tipo, protocolo, família, endereço e porta | Colchetes em IPv6 ajudam a distinguir endereço de porta; não incluir credenciais/SDP bruto |
| `enviar_rtp` | Lê RTP local, filtra payload types, ajusta SSRC e escreve na trilha | Preservar coerência entre parâmetros da trilha e pacotes do encoder |
| `receber_rtp` | Lê trilha remota, adapta cabeçalho e envia ao decoder local | Não confundir RTP codificado com quadro RGBA pronto |

`watch` serve a sinais cujo valor atual basta, como coleta concluída ou parada.
`mpsc` serve a uma sequência de eventos. O canal de mudanças é sem limite,
enquanto os canais de mídia/API são pequenos. Isso não autoriza enviar frames
no canal sem limite: os frames usam o caminho limitado para evitar fila crescente.

As tarefas RTP encerram seus laços se leitura/escrita falhar. Algumas dessas
saídas não produzem um erro específico para a interface. A vigilância detecta
estado da conexão e morte de helpers, mas não mede ausência contínua de pacotes.
Essa é uma limitação de observabilidade, útil para planejar um próximo incremento.

## rust/tests/ipv6.rs: transporte sem codec real

[ipv6.rs](../../../rust/tests/ipv6.rs) monta PeerConnections próprias. Não chama
todo o fluxo de `chamada::executar`, não usa o servidor de salas e não abre FFmpeg.
O helper `criar` registra codecs e trilhas; `ensaiar` troca descrições diretamente,
envia o payload conhecido `payload-ipv6`, confere recepção e consulta o par ICE.

O SSRC vem de `trilha.ssrcs()`. O payload conhecido não é um bitstream VP8/Opus
decodificável: o teste comprova transporte das trilhas, não imagem ou som.

| Teste | O que exercita |
| --- | --- |
| `rtp_de_audio_e_video_bidirecional_em_ipv6_com_repeticao` | Duas rodadas em `::1`, trilhas nos dois sentidos e par host IPv6 |
| `runtime_ice_resolve_hostname_para_ipv6` | Resolução de `localhost` pelo runtime utilizado pela biblioteca |
| `stun_por_hostname_coleta_candidato_srflx_ipv6` | Resposta STUN controlada e inclusão de srflx IPv6 no SDP |
| `rtp_bidirecional_por_turn_ipv6_real` | Ensaio ignorado por padrão; depende de coturn configurado e exige relay IPv6 local/remoto |

O endereço `2001:db8::20` usado pelo falso STUN é de documentação. A resposta
foi construída pelo teste; não demonstra NAT ou conectividade pública real.

## rust/tests/chamada_sintetica.rs: o caminho completo sem dispositivos

[chamada_sintetica.rs](../../../rust/tests/chamada_sintetica.rs) inicia a
sinalização em loopback e chama `executar` duas vezes, com `sintetica=true`.
São duas sessões no mesmo processo, acessando diretamente a camada interna;
a restrição de uma sessão da API Flutter não é usada pelo teste.

Ele exige cinco quadros remotos RGBA e cinco blocos PCM em cada participante,
além do evento de caminho ICE. Cancela os dois, drena os canais durante a
limpeza, aguarda término e repete a chamada. A segunda rodada ajuda a detectar
recursos não liberados e problemas na reutilização da sala.

A expectativa do texto ICE é `direto` quando não força TURN e `retransmissão
TURN` quando força. Isso é uma hipótese do ensaio local: política `All` no
produto pode legitimamente escolher TURN em outra rede. Esse teste sozinho
não exige relay↔relay IPv6; o teste TURN de `ipv6.rs` verifica esses detalhes.

O teste é ignorado por padrão porque depende dos executáveis de mídia.
Os [comandos e pré-requisitos](../../desenvolvimento/testes.md) ficam em
desenvolvimento. `diagnosticar_midia.rs` é explicado no [módulo de mídia](../midia/arquivos.md),
e `api.rs`/bindings no [módulo da ponte](../ponte-flutter-rust/arquivos.md).
