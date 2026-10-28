# Documentação da POC

Há dois caminhos de leitura. Escolha pelo que você precisa fazer agora:

| Quero… | Começar por |
| --- | --- |
| Preparar o ambiente, executar e distribuir a POC | [Desenvolvimento e testes](desenvolvimento/README.md) |
| Saber o que foi concluído e o que ainda precisa ser validado | [Etapas](desenvolvimento/etapas.md) e [evidências](desenvolvimento/validacao.md) |
| Entender as tecnologias e acompanhar o código | [Trilha de estudos](estudos/README.md) |
| Estudar a integração de Rust com WebRTC | [Módulo webrtc-rs](estudos/webrtc/README.md) |
| Investigar uma falha na chamada | [Guia de testes](desenvolvimento/testes.md) e [diagnóstico da primeira chamada](desenvolvimento/diagnostico-primeira-chamada.md) |

**Desenvolvimento** reúne procedimentos, critérios de aceite e registros datados.
**Estudos** explica responsabilidades, decisões e arquivos da implementação.
Os dois conjuntos se referenciam: a explicação de um teste fica junto ao módulo,
mas seus comandos e resultados ficam na documentação de desenvolvimento.

Os estudos descrevem o código inspecionado em **27/10/2026**, incluindo as
alterações locais existentes. Os registros de execução mantêm suas próprias
datas; esta reorganização não representa uma nova execução da POC.

## Onde foram parar os documentos anteriores

| Caminho anterior em docs/ | Destino |
| --- | --- |
| 01-arquitetura.md | [Estudos / arquitetura](estudos/arquitetura/README.md) |
| 02-etapas.md | [Desenvolvimento / etapas](desenvolvimento/etapas.md) |
| 03-guia-do-codigo.md | [Índice dos estudos](estudos/README.md), aprofundado nos módulos |
| 04-validacao.md | [Desenvolvimento / validação](desenvolvimento/validacao.md) |
| 05-etapa-1.md | [Desenvolvimento / Etapa 1 LAN](desenvolvimento/etapa-1-lan.md) |
| 06-etapa-2.md | [Desenvolvimento / configuração da Etapa 2](desenvolvimento/etapa-2-configuracao.md) |
| 07-roteiro-chamada-entre-redes.md | [Desenvolvimento / roteiro entre redes](desenvolvimento/roteiro-entre-redes.md) |
| 08-auditoria-ipv6.md | [Desenvolvimento / auditoria IPv6](desenvolvimento/auditoria-ipv6.md) |
| 09-diagnostico-primeira-chamada.md | [Desenvolvimento / diagnóstico](desenvolvimento/diagnostico-primeira-chamada.md) |

## Como manter a documentação coerente

Quando mudar um contrato ou uma função, atualize o módulo de estudo responsável.
Quando mudar um comando de execução, atualize o guia operacional. Quando executar
um ensaio, acrescente data, condições, resultado e limites ao histórico de
validação. Uma instrução preparada não deve ser registrada como um teste aprovado.

Use links relativos entre documentos e código. Versões fixadas e comportamento
da POC têm prioridade sobre exemplos de outras versões das bibliotecas. Evite
copiar o mesmo procedimento para vários módulos: explique o motivo e aponte
para o procedimento mantido em desenvolvimento.
