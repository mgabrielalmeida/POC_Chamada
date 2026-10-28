# Desenvolvimento e testes

[Documentação](../README.md) · [Estudos das tecnologias e do código](../estudos/README.md)

Esta pasta responde como preparar, executar, testar e acompanhar a POC. Para
aprender por que o código está organizado dessa forma, siga os módulos de estudos.

## Ordem de uso

1. Leia [etapas e critérios](etapas.md) para conhecer o escopo de cada entrega.
2. Use [Etapa 1 — LAN](etapa-1-lan.md) para preparar FRB, mídia e os dois Windows.
3. Consulte [testes](testes.md) para escolher uma verificação adequada à mudança.
4. Para internet, leia [configuração da Etapa 2](etapa-2-configuracao.md) e execute
   o [roteiro entre redes](roteiro-entre-redes.md), separado por Windows A, B e S.
5. Registre o que ocorreu em [validação](validacao.md). Leia os registros datados
   em ordem: uma pendência antiga pode ter sido investigada em uma rodada posterior.

## Situação documentada

A Etapa 1 tem confirmação humana de chamada entre dois Windows na mesma rede em
21/10/2026. A Etapa 2 permanece sem aceite físico completo. Há registros de WSS
acessível na tentativa humana e de transporte/mídia sintética por TURN no próprio
servidor S, após correções em 23/10/2026. Isso não substitui cinco minutos de
áudio/vídeo reais entre A/B em redes distintas, nos modos normal e TURN forçado,
com encerramento e repetição. Veja o [diagnóstico mais recente](diagnostico-primeira-chamada.md).

Essas são evidências históricas; não indicam que os serviços estejam ativos hoje.

## Referências de investigação

| Documento | Quando consultar |
| --- | --- |
| [Validação](validacao.md) | Comandos executados, versões observadas e limitações de cada rodada |
| [Auditoria IPv6](auditoria-ipv6.md) | Inspeção da versão fixada, alterações IPv6 e ensaio TURN isolado |
| [Diagnóstico da primeira chamada](diagnostico-primeira-chamada.md) | Falhas dos helpers, coleta ICE, filtros coturn e correções verificadas |
| [Dificuldades do WebRTC](../estudos/webrtc/dificuldades-e-decisoes.md) | Explicação didática das causas e das soluções |

Os dois relatórios datados permanecem aqui porque documentam investigações e
evidências. O aprendizado que pode ser reutilizado foi organizado nos estudos.
