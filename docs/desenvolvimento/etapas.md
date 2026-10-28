# Etapas e critérios de conclusão

O prazo é curto: cada passo deve aproximar uma chamada real. Os checkpoints são
marcos funcionais; commits intermediários menores são permitidos e recomendados.

[Desenvolvimento](README.md) · [Estudos](../estudos/README.md)

## Base — estrutura do projeto

Entregar Flutter para Windows, workspace Rust com biblioteca cliente e servidor,
Git local e documentação inicial. Registrar os comandos realmente executados.
Esta entrega não representa uma chamada ou integração entre linguagens.

## Etapa 1 — dois computadores na mesma rede

**Concluída em 21/10/2026.** O responsável pelo teste confirmou o funcionamento
entre dois dispositivos Windows na mesma rede Wi-Fi. A implementação, as
verificações automatizadas e o relato de aceitação estão em
[validação](validacao.md).

Ordem de trabalho sugerida:

1. Integrar Flutter-Rust Bridge e comprovar uma chamada simples Dart → Rust.
2. Verificar o caminho mínimo de captura, codecs e reprodução no Windows antes
   de ampliar a interface; fixar versões e documentar dependências nativas.
3. Implementar sinalização Rust com salas de duas pessoas e negociação WebRTC.
4. Integrar áudio e vídeo de ponta a ponta com webrtc-rs.
5. Encerrar corretamente e documentar a execução em dois computadores.

Não criar todas as pastas e abstrações antecipadamente. Acrescentar módulos
conforme funções reais surgirem. Se um risco de mídia bloquear o fluxo, resolvê-lo
antes de investir em interface ou infraestrutura da segunda etapa.

**Aceitação humana em dois dispositivos físicos:**

- Ambos enviam e recebem áudio e vídeo durante pelo menos cinco minutos.
- A tela apresenta vídeo local, vídeo remoto e estado compreensível da chamada.
- Encerrar libera câmera e microfone; uma nova chamada funciona.
- O procedimento pode ser repetido seguindo a documentação.

Registrar resultado, versões, equipamentos, problemas e limitações observadas.
Build, logs e testes automatizados não substituem essa aceitação.

## Etapa 2 — computadores em redes diferentes

**Em implementação, sem aceite físico completo.** Cliente com STUN/TURN
configurável, diagnóstico ICE e sinalização com token estão implementados.
A topologia documentada usa um terceiro Windows doméstico com IPv6 público,
CGNAT IPv4 e sem VPN. O [registro de 23/10](diagnostico-primeira-chamada.md)
descreve serviços instalados, sinalização acessível na tentativa humana e
ensaios sintéticos por TURN no próprio S após correções. Não comprova os
cenários físicos completos entre A/B nem disponibilidade atual dos serviços.
Consulte o [roteiro de execução](roteiro-entre-redes.md) e o
[guia de configuração](etapa-2-configuracao.md) para preparar ou repetir o ensaio.

1. Conferir IPv6 global de S e conectividade IPv6 dos dois participantes externos.
2. Preparar o terceiro Windows e configurar domínio/AAAA ou DDNS IPv6.
3. Preparar sinalização com token e pacote coturn nativo Windows.
4. Autorizar entrada IPv6 TCP 80/443 e UDP 3478/49160–49200 nos firewalls Windows/roteador.
5. Ativar Caddy/WSS com certificado público e verificar acesso de uma rede externa.
6. Configurar os participantes e executar chamada normal e TURN forçado, com cinco minutos, encerramento e repetição.
7. Registrar versões, endereços e comandos reais sem segredos; verificar reinício automático se configurado.

Registrar o par selecionado em A/B, com tipo, família, endereço e porta.
No TURN forçado, exigir relay IPv6 local/remoto em ambos, contadores do coturn
e áudio/vídeo bidirecional durante cinco minutos; encerrar e repetir.
Registrar se a conexão usou caminho direto ou retransmissão. Para afirmar que o
TURN funciona, realizar também um teste que force retransmissão e verificar o
caminho escolhido; uma chamada direta não comprova esse serviço.

## Disciplina de entrega

- Commits locais pequenos, com mensagem clara; sem envio a repositório remoto.
- Manter README, guias operacionais e módulos de estudos coerentes com a implementação.
- Registrar evidências e pendências em [validação](validacao.md) a cada checkpoint.
- Priorizar comandos reproduzíveis; criar scripts apenas quando reduzirem
  repetição ou erros reais de execução.
