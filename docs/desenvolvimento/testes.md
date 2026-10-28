# Como escolher e executar os testes

[Desenvolvimento](README.md) · [Histórico de resultados](validacao.md)

Os comandos abaixo são procedimentos, não resultados desta revisão documental.
Execute em PowerShell na raiz do checkout, exceto onde `Push-Location app`
indica a troca de pasta. Ambiente e preparação inicial estão no
[guia LAN](etapa-1-lan.md). Feche instâncias que estejam usando a DLL antes de recompilar.

## 1. Conferir código e configuração

```powershell
cargo fmt --all -- --check
cargo check --locked --workspace
cargo test --locked --workspace
git diff --check
Push-Location app
dart format --output=none --set-exit-if-changed lib tool
flutter analyze
Pop-Location
```

A suíte Rust comum inclui configuração, WebSockets reais em loopback e testes
IPv6 locais. Não é puramente uma análise estática. Requer disponibilidade de
sockets locais IPv4/IPv6; bloqueios do ambiente de execução podem impedir o
teste sem significar defeito no protocolo.

Os ensaios de FFmpeg e coturn permanecem ignorados na suíte comum. Consulte os
pré-requisitos abaixo antes de executá-los, em vez de usar `--ignored` para
todos os testes do workspace de uma vez.

## 2. Conferir a ponte sem dispositivos

```powershell
cargo build --locked -p chamada
Push-Location app
dart run tool/verificar_ponte.dart
Pop-Location
```

Deve confirmar mensagem simples, stream, erro assíncrono e encerramento.
Não abre câmera ou microfone. Uma alteração de API pública exige regenerar
bindings conforme o [guia LAN](etapa-1-lan.md) antes desse teste.

## 3. Isolar captura real

Depois de preparar os helpers:

```powershell
cargo run --locked -p chamada --example diagnosticar_midia
```

Abre a primeira câmera/microfone DirectShow, exige um quadro RGBA e um RTP Opus,
e libera recursos. Evite outro aplicativo usando os mesmos dispositivos.
O avanço esperado é obter as duas evidências e o encerramento normal; isso
não mede áudio remoto, qualidade percebida ou acesso pela internet.

## 4. Conferir transporte e decodificação sintéticos

```powershell
cargo test --locked -p chamada --test chamada_sintetica -- --ignored --nocapture
```

Exige FFmpeg preparado, inicia sinalização local e duas sessões, usa vídeo e
tom gerados e não toca som. Cada rodada exige pelo menos cinco quadros remotos,
cinco blocos PCM por participante e diagnóstico ICE; encerra e repete.

O teste herda `CHAMADA_CONFIG`. Para uma regressão local sem serviços externos,
use um terminal sem essa variável. Para ensaiar Relay, aponte-a ao JSON privado
preparado para o serviço TURN autorizado. Não copie segredos para o comando ou
para o relatório.

No modo normal, o teste espera texto `direto`; na aplicação, política `All`
também pode escolher TURN. Portanto essa asserção é específica do cenário
local esperado pelo teste. Para comprovar família e tipos de ambos os candidatos,
use também o teste de transporte TURN IPv6 e confira o diagnóstico.

## 5. Conferir IPv6 e TURN

Os testes locais, sem coturn, já fazem parte da suíte comum. Para isolá-los:

```powershell
cargo test --locked -p chamada --test ipv6
```

O ensaio ignorado `rtp_bidirecional_por_turn_ipv6_real` exige um processo
coturn e variáveis `CHAMADA_TESTE_TURN_PORTA`, `CHAMADA_TESTE_TURN_USUARIO` e
`CHAMADA_TESTE_TURN_SENHA`. O procedimento completo de teste **isolado em
loopback** está na [auditoria IPv6](auditoria-ipv6.md). HOST e BIND opcionais
são descritos no [diagnóstico posterior](diagnostico-primeira-chamada.md).

Esse ensaio exige relay local/remoto IPv6 e payload conhecido nas duas trilhas
dos dois peers. Não decodifica vídeo/áudio nem inicia coturn automaticamente.
Use o ensaio sintético para acrescentar decodificação e o físico para validar
as redes e os dispositivos reais.

## 6. Compilar e aceitar em máquinas reais

```powershell
cargo build --locked -p sinalizacao --release
Push-Location app
flutter build windows --release
Pop-Location
```

Distribua a pasta Release inteira conforme [Etapa 1](etapa-1-lan.md).
O build não executa o aceite. Na LAN, siga os critérios da Etapa 1; em redes
distintas, siga os cinco cenários do [roteiro](roteiro-entre-redes.md).

| Evidência | Comprova | Não comprova |
| --- | --- | --- |
| Check/análise/build | Coerência estática e produção de artefatos | Execução real de chamada |
| Ponte | Travessia Dart/DLL, stream e liberação | Captura ou rede |
| Servidor em loopback | Contrato de salas e autorização | WSS público e mídia |
| RTP IPv6 com payload conhecido | Transporte e par selecionado | Codec/qualidade audiovisual |
| Captura real isolada | Dispositivo e saída inicial dos encoders | Recepção na outra máquina |
| Sintético com FFmpeg | Transporte, decodificação e repetição local | Câmera/microfone reais ou fronteiras A/B |
| Ensaio físico normal e forçado | Comportamento observado naquelas redes e dispositivos | Garantia em qualquer rede ou prontidão de produção |

## Registrar o resultado

Acrescente data, versão/commit ou identificação do checkout, máquinas/redes,
comandos, política ICE, par observado, duração, contadores e problemas.
Em caso de falha, guarde o nome do helper e stderr relevante. Remova tokens,
senhas e dados privados de SDP antes de versionar o relato.

Não substitua evidências antigas para fazer o histórico parecer linear.
Registre a correção e a nova execução, indicando o que ela comprovou e o que
permanece pendente. Para revisão exclusivamente documental, conferir links,
caminhos, contratos e sintaxe dos comandos é mais pertinente do que abrir
dispositivos ou alterar serviços.
