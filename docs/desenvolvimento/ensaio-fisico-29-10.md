# Ensaio físico A/B pela internet — 29/10/2026

[Desenvolvimento](README.md) · [Diagnóstico anterior](diagnostico-primeira-chamada.md) · [Validação](validacao.md)

Registro do ensaio humano executado com o [roteiro entre redes](roteiro-entre-redes.md)
após as correções de 23/10. As observações abaixo foram relatadas pelo responsável;
as causas são **hipóteses** levantadas pela leitura do código e da configuração,
ainda não confirmadas por medição. Nenhum código foi alterado nesta rodada.

## Ambiente

- **Windows S:** sinalização (`127.0.0.1:8081`), Caddy e coturn 4.18.0 iniciados
  pelos lançadores de `C:\POC_Servidor`. S estava conectado à internet por
  hotspot de celular (perfil de rede `Public`), com AAAA em `poc-chamada.dynv6.net`.
- **Windows A e B:** pacote `POC_Chamada_Etapa2_Corrigido` (versão 20261023-134427),
  em redes diferentes, conforme o roteiro. No A, a pasta extraída precisou de
  `Unblock-File` antes de `liberar-aplicativo.ps1`: o Windows bloqueava o script
  não assinado vindo do ZIP sob a política `RemoteSigned`.
- Havia uma sessão AnyDesk entre B e S durante a preparação. Não foi registrado
  se ela permaneceu aberta durante as chamadas.
- Não foram registrados o texto do `Caminho ICE`, a duração exata de cada
  chamada nem os arquivos `.stderr.log` dos participantes.

## Resultados observados

| # | Modo | Criou a sala | Vídeo | Áudio A → B | Áudio B → A |
| --- | --- | --- | --- | --- | --- |
| 1 | Normal | A | Bom nos dois sentidos, pouco atraso | Chega com ~30 s de atraso e pouca perda de qualidade | Nunca chega |
| 2 | Normal | B | Igual ao nº 1 | Igual ao nº 1 | Nunca chega |
| 3 | TURN forçado | A | Chega nos dois sentidos, com qualidade baixa, fps baixo e travamentos ocasionais | Mesmo atraso, com mais perda de qualidade | Não chega; ruídos ocasionais aleatórios |

O que funcionou: WSS autenticado pelo domínio IPv6, entrada na sala nos dois
papéis, conexão WebRTC nos modos normal e TURN forçado, e vídeo bidirecional.
Uma segunda chamada no modo normal também conectou (nº 1 → nº 2). Não foi
registrado se os serviços de S ficaram ativos sem reinício entre as chamadas.

**Situação do aceite:** a Etapa 2 continua **sem aceite**. Os critérios de
áudio bidirecional não foram atendidos em nenhum modo, e o par ICE de cada
chamada não foi registrado.

## Análise das causas prováveis

### 1. Atraso de ~30 s no áudio (A → B, nos dois modos)

**Hipótese principal: áudio acumulado antes da conexão.**

1. `Midia::iniciar` é chamado logo depois de o servidor responder `pronto`
   ([chamada.rs](../../rust/src/chamada.rs)). Nesse momento, o FFmpeg do
   microfone já começa a capturar e a enviar RTP para o socket UDP de loopback
   `midia.audio` ([midia.rs](../../rust/src/midia.rs)).
2. Esse socket só passa a ser lido por `enviar_rtp` quando o estado vira
   `Connected`. Antes disso, a coleta ICE pode levar até 10 s, e oferta,
   resposta e conectividade ainda somam mais tempo. Enquanto isso, os pacotes
   de áudio se acumulam no buffer de recepção do socket no Windows.
3. Na conexão, todo o acúmulo é enviado de uma vez. O `ffplay` do outro lado
   reproduz o áudio no ritmo real, a partir do pacote mais antigo. O atraso
   inicial vira **permanente**, porque nada descarta o excesso: não há jitter
   buffer adaptativo nem descarte por idade.
4. O vídeo sofre o mesmo acúmulo, mas o receptor de vídeo usa
   `-fps_mode passthrough` e a interface descarta quadros. Ele esvazia o
   acúmulo rápido e volta ao tempo real. Isso explica o vídeo com pouco atraso
   e o áudio com muito atraso na mesma chamada.

O Opus a 32 kbit/s gera pacotes pequenos (50 por segundo). Um buffer de recepção
de dezenas de KB guarda vários segundos desses pacotes, enquanto guarda apenas
1–2 s do vídeo a 300 kbit/s. O valor real do buffer no Windows não foi medido.

Hipóteses secundárias, menos prováveis para um atraso dessa ordem: buffer de
saída do `ffplay`/SDL e o `-audio_buffer_size 50` do DirectShow. Essas causas
explicariam centenas de milissegundos, não dezenas de segundos.

Os testes sintéticos (`chamada_sintetica`, `ipv6`) confirmam que PCM foi
decodificado, mas **não medem latência**. Por isso não detectaram o problema.

**Como confirmar:** comparar o atraso percebido com o tempo entre
"Preparando câmera, microfone e WebRTC…" e "WebRTC conectado" em cada lado.
Se a hipótese estiver certa, chamadas que negociam mais rápido terão menos
atraso. Outra forma: contar quantos pacotes `enviar_rtp` lê do socket no
primeiro segundo após a conexão.

**Direções de correção, não implementadas:** começar a ler o socket e descartar
os pacotes enquanto não houver conexão, iniciar o microfone só depois de
`Connected`, ou descartar o acúmulo no momento da conexão.

### 2. Áudio de B não chega ao A (nos dois modos e nos dois papéis)

O defeito é igual com A ou com B criando a sala. Como o código de envio e de
recepção é o mesmo nas duas máquinas, e o caminho A → B funciona, a causa mais
provável está no **ambiente de B** (captura) ou, menos provável, no de A
(reprodução).

Hipóteses, da mais à menos provável:

1. **Dispositivo errado ou silencioso no B.** `dispositivos()` escolhe o
   **primeiro** dispositivo de áudio DirectShow listado, sem tela de seleção.
   No B, ele pode ser um dispositivo virtual, uma entrada sem sinal (por exemplo,
   "Mixagem estéreo") ou um microfone mudo ou com volume zero.
2. **Privacidade do Windows no B.** Com "Permitir que aplicativos da área de
   trabalho acessem o microfone" desativado, a captura pode entregar silêncio em
   vez de erro. O helper continua vivo e nenhuma mensagem aparece, o que bate
   com o que foi observado.
3. **Uso do microfone por outro programa no B**, como a sessão AnyDesk ou um
   aplicativo de comunicação, com modo exclusivo habilitado.
4. **Saída de áudio do `ffplay` no A** num dispositivo diferente dos fones.
   É pouco provável, porque A ouviu ruídos no modo TURN, o que indica que a
   reprodução no A funciona.

Os ruídos do modo TURN são compatíveis com um fluxo Opus quase silencioso
somado a perdas de pacote: o decodificador gera artefatos de ocultação de perda.
Isso sugere que B **envia** áudio, mas de uma fonte sem voz útil. Também é
possível que fosse ruído de fundo do dispositivo errado.

Uma falha de captura que encerrasse o helper apareceria como erro de
`microfone` na interface (`verificar_processos`). O fato de nenhuma mensagem ter
sido relatada reforça as hipóteses 1 e 2 contra uma falha de abertura do
dispositivo.

**Como confirmar no B**, em PowerShell normal na pasta `Release` do pacote:

```powershell
.\ffmpeg.exe -hide_banner -list_devices true -f dshow -i dummy
.\ffmpeg.exe -hide_banner -f dshow -i audio="NOME_DO_PRIMEIRO_DISPOSITIVO" -t 5 teste-microfone.wav
.\ffplay.exe -autoexit teste-microfone.wav
```

O primeiro dispositivo `(audio)` da lista é o que a POC usa. Se a gravação sair
muda, o problema está confirmado no B. Também conferir a privacidade do
microfone e o `.stderr.log` mais recente da pasta `logs` dos dois participantes.
No modo TURN, os contadores do coturn por alocação permitem comparar o volume
enviado por A e por B.

### 3. Vídeo e áudio piores no TURN forçado

1. **Upload do hotspot de S.** No modo relay, toda a mídia passa por S: ele
   recebe e reenvia cerca de 0,35 Mbit/s por sentido (vídeo a 300 kbit/s, Opus a
   32 kbit/s e cabeçalhos). Isso dá cerca de 0,7 Mbit/s de entrada e 0,7 Mbit/s
   de saída, com a mesma rede móvel. O uplink celular varia bastante, e uma
   sessão AnyDesk aberta disputaria a mesma banda.
2. **Ausência de adaptação.** O VP8 sai com bitrate fixo de 300 kbit/s. Os
   interceptores padrão do webrtc-rs geram NACK e relatórios RTCP, mas nenhum
   pedido de quadro-chave ou estimativa de banda volta ao FFmpeg. Uma perda
   corrompe o vídeo até o próximo quadro-chave (`-g 10` a 10 fps, cerca de 1 s),
   o que aparece como travamentos.
3. **Opus sem FEC.** O áudio não usa correção de erro embutida
   (`-fec`/`-packet_loss`). Por isso as perdas do relay aparecem diretamente
   como piora de qualidade.
4. O vídeo é sempre 320×240 a 10 fps. Em comparação com o modo normal, a perda
   no relay ficou visível.

**Como confirmar:** registrar o `Caminho ICE` nos dois modos e os contadores de
bytes do coturn. Repetir o modo TURN com o AnyDesk fechado. Se possível, testar
S numa conexão fixa em vez do hotspot.

## Pendências para o próximo ensaio

1. Confirmar no B qual microfone é usado e se ele grava áudio (seção 2).
2. Registrar o `Caminho ICE` de A e B em cada chamada, além da duração.
3. Guardar os `.stderr.log` de A e B e o log do coturn de cada chamada, fora do Git.
4. Medir o atraso do áudio contra o tempo de negociação (seção 1).
5. Repetir o modo TURN sem AnyDesk e, se possível, com S fora do hotspot.
6. Só então decidir as correções de código (descarte do acúmulo de áudio,
   seleção de microfone e eventual FEC), que ficaram fora desta rodada.
