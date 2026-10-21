# Etapas e critérios de conclusão

O prazo é curto: cada passo deve aproximar uma chamada real. Os checkpoints são
marcos funcionais; commits intermediários menores são permitidos e recomendados.

## Base — estrutura do projeto

Entregar Flutter para Windows, workspace Rust com biblioteca cliente e servidor,
Git local e documentação inicial. Registrar os comandos realmente executados.
Esta entrega não representa uma chamada ou integração entre linguagens.

## Etapa 1 — dois computadores na mesma rede

Ponte e sinalização implementadas. A integração de mídia e sua verificação estão
registradas em [validação](04-validacao.md); a aceitação humana abaixo permanece
obrigatória antes de considerar a etapa concluída.

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

1. Inspecionar a máquina disponível: sistema, administração e acesso pela internet.
2. Disponibilizar sinalização com configuração explícita e acesso controlado.
3. Preparar TURN próprio e configurar os clientes Rust quando necessário.
4. Documentar endereços, portas e comandos reais, sem versionar segredos.
5. Repetir a aceitação de cinco minutos, encerramento e nova chamada nas duas redes.

Registrar se a conexão usou caminho direto ou retransmissão. Para afirmar que o
TURN funciona, realizar também um teste que force retransmissão e verificar o
caminho escolhido; uma chamada direta não comprova esse serviço.

## Disciplina de entrega

- Commits locais pequenos, com mensagem clara; sem envio a repositório remoto.
- Manter README e guia de código coerentes com a implementação atual.
- Registrar evidências e pendências em `04-validacao.md` a cada checkpoint.
- Priorizar comandos reproduzíveis; criar scripts apenas quando reduzirem
  repetição ou erros reais de execução.
