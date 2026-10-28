# Diagnóstico da primeira chamada pela internet

[Desenvolvimento](README.md) · [Explicação das causas e soluções](../estudos/webrtc/dificuldades-e-decisoes.md)

Data: 23/10/2026. Investigação após a tentativa humana entre Windows A e B.
Este registro complementa a auditoria anterior; o computador desta investigação
é o Windows S, com sinalização, Caddy e coturn já instalados.

## Sintomas e evidências

- A sala foi criada e o segundo cliente entrou: a sinalização estava acessível.
- As câmeras ligaram. O A mostrou `exit code: 0xffffffea`; o B mostrou
  `Tempo esgotado ao reunir os endereços ICE: deadline has elapsed`.
- `0xffffffea` é a representação Windows de `-22`, erro de argumento inválido
  do FFmpeg. O código isolado não identifica o helper que encerrou.
- O receptor VP8 foi reproduzido sem RTP, sem abrir câmera: terminou com `-22`
  e `Output file does not contain any stream`. O demuxer SDP instalado usa
  `listen_timeout=10` por padrão. Os receptores eram iniciados antes de uma
  negociação que podia consumir 30 s na oferta, 30 s na resposta e mais tempo
  para conectar o transporte. Isso reproduz o sintoma do A, mas não substitui
  o stderr daquela máquina para confirmar a origem da execução específica.
- O log do coturn real registrou peers IPv6 públicos rejeitados pela faixa
  `::`. Em `ioa_addr_in_range`, esse endereço é considerado limite curinga;
  `denied-peer-ip=::` acabava bloqueando todos os peers. A mesma linha estava
  no modelo usado na implantação.
- A origem exata da coleta incompleta no B não foi identificada remotamente.
  Uma interface/servidor sem resposta podia fazer o cliente descartar todos os
  candidatos que outras interfaces já haviam reunido. A rejeição TURN é outro
  impedimento confirmado para retransmissão, depois da coleta de candidatos.

## Correções

1. Retirada apenas a regra `denied-peer-ip=::` do modelo e da configuração real.
   `good_peer_addr` já rejeita endereços não especificados; autenticação, quotas,
   bloqueios privados, link-local, multicast e loopback permanecem. Não adicionar
   uma whitelist global nem habilitar `allow-loopback-peers` no serviço público.
2. Receptores de vídeo e áudio usam `-listen_timeout 120`. Rust continua
   encerrando os helpers no cancelamento e na falha de conexão; o prazo do
   transporte não foi removido.
3. A espera de ICE passa a ser limitada a 10 s, publica a prévia local e
   verifica os processos durante esse período. Ao atingir o limite, envia o
   snapshot SDP dos candidatos já disponíveis. Candidatos posteriores ficam
   fora dessa negociação, que continua sem trickle ICE. Esperar 30 s antes da resposta também podia esgotar o prazo interno do ICE remoto. Não há garantia de
   sucesso se nenhum desses candidatos for alcançável.
4. Sem candidatos, a negociação falha com orientação de DNS/UDP. No modo TURN
   forçado, exige pelo menos um candidato relay antes de enviar o SDP; não
   permite substituir a exigência de relay por candidatos host.
5. Cada helper tem um nome e um histórico limitado das últimas oito linhas de
   stderr, incluído no erro de encerramento. Os lançadores do pacote atualizado
   salvam stdout/stderr por execução em `logs`, sem copiar o JSON de credenciais.
6. Flutter espelha horizontalmente apenas a imagem de “Seu vídeo”. O quadro
   transmitido e a imagem da outra pessoa preservam a orientação original.

## Verificações e limites

- Antes: receptor SDP sem RTP encerrou com `-22`. Depois: continuou vivo após
  13 s sem RTP e decodificou cinco quadros RGBA, totalizando 1.536.000 bytes,
  terminando com código zero.
- Ensaio existente `ipv6::rtp_bidirecional_por_turn_ipv6_real` passou pelo
  coturn real corrigido, usando seu IPv6 global e hostname. Ambos os peers
  selecionaram relay local/remoto e receberam os payloads RTP esperados de
  áudio e vídeo. É uma execução no próprio S; não prova travessia das redes A/B.
- A suíte IPv6 local passou: três testes; o ensaio TURN é ignorado por padrão
  e foi executado separadamente com credenciais privadas.
- O teste sintético existente de duas chamadas seguidas passou com a
  configuração normal: vídeo e PCM remotos decodificados nos dois participantes.
- Com servidor TURN de descarte que não responde, a coleta atingiu o limite
  de 10 s em cada peer. Duas chamadas seguidas conectaram por IPv6 e
  decodificaram vídeo/PCM nos dois sentidos; o ensaio terminou em 42,75 s.
  A versão intermediária com 30 s chegou a conectar uma chamada, mas falhou
  na repetição pelo prazo interno do ICE; por isso não foi distribuída.
- O ensaio sintético também passou duas vezes seguidas com a configuração
  TURN forçada real: ambos os lados selecionaram relay local/remoto IPv6 e
  decodificaram vídeo e PCM. Duração do ensaio: 11,78 s. O teste agora confere
  o caminho esperado conforme a política do JSON, sem imprimir credenciais.
- Build Windows Release e análise Flutter concluídos. Os binários dentro do
  ZIP de atualização foram comparados por SHA-256 aos arquivos do build.
- A aceitação de câmera/microfone reais, qualidade audiovisual, duração de cinco
  minutos e repetição entre A/B continua dependendo de novo ensaio humano.

O ensaio TURN aceita opcionalmente `CHAMADA_TESTE_TURN_HOST` e
`CHAMADA_TESTE_TURN_BIND`; seus padrões continuam `localhost` e `[::1]:0`.
Não usar credenciais reais em arquivos versionados ou na linha de comando.

Referências: [coturn 4.18.0: comparação de faixas](https://github.com/coturn/coturn/blob/4.18.0/src/client/ns_turn_ioaddr.c),
[coturn 4.18.0: validação de peers](https://github.com/coturn/coturn/blob/4.18.0/src/server/ns_turn_server.c),
[FFmpeg: opção SDP listen_timeout](https://ffmpeg.org/doxygen/trunk/rtsp_8c_source.html),
[Flutter: Transform.flip](https://api.flutter.dev/flutter/widgets/Transform/Transform.flip.html).
