# Ponte: leitura arquivo a arquivo

[Funcionamento geral](README.md) · [Procedimento de geração](../../desenvolvimento/etapa-1-lan.md)

## rust/src/api.rs

[api.rs](../../../rust/src/api.rs) é a fonte manual do contrato exposto.
`EventoChamada` contém texto e bytes; `Sessao` é privada e contém o remetente
`watch` de parada e o `JoinHandle` da tarefa. O gerador não precisa expor essa
estrutura interna ao Dart.

`observar_chamada` envia `pronta` ao novo sink e o guarda em `OBSERVADOR`.
Registrar um novo observador substitui o anterior, coerente com uma janela
por processo. `encerrar_observacao` retira o sink do `Option` com `take()`.

`iniciar_chamada` primeiro trava `SESSAO`. Se a tarefa atual não terminou,
retorna erro. Se existe tarefa anterior já finalizada, retira e aguarda seu
handle. Depois cria `watch(false)` para parada e `mpsc(8)` para eventos.

Há uma tarefa externa que encaminha eventos ao sink e uma tarefa interna que
executa `chamada::executar`. A externa consome o canal até ele fechar e espera
a interna. Quando não consegue entregar um evento ao observador, envia parada.
O modo passado a `executar` é `sintetica=false`: a API da interface sempre usa
dispositivos reais.

`encerrar_chamada` retira a sessão, envia `true` no canal de parada e aguarda
a tarefa. Manter o mutex durante esse processo impede uma entrada concorrente
antes da liberação. `verificar_ponte` retorna texto fixo, sem abrir rede ou mídia.

## app/flutter_rust_bridge.yaml

A [configuração do gerador](../../../app/flutter_rust_bridge.yaml) seleciona
`crate::api`, aponta `rust_root` para `../rust/` e `dart_output` para
`lib/src/rust`. `web=false` deixa esse alvo fora da geração; não é uma promessa
de compatibilidade do cliente Windows com navegador. A atualização automática
de dependências fica desativada.

Gerar bindings e compilar a DLL são operações diferentes. O gerador cria o
contrato de interoperabilidade; Cargo produz o binário que o executa. O
[guia da Etapa 1](../../desenvolvimento/etapa-1-lan.md) reúne os comandos na ordem.

## Arquivos gerados

| Arquivo | Papel na travessia |
| --- | --- |
| [rust/src/frb_generated.rs](../../../rust/src/frb_generated.rs) | Entrada gerada do lado Rust, despacho e conversão dos valores/streams |
| [app/lib/src/rust/api.dart](../../../app/lib/src/rust/api.dart) | Fachada tipada em Dart: funções públicas e `EventoChamada` com `Uint8List` |
| [app/lib/src/rust/frb_generated.dart](../../../app/lib/src/rust/frb_generated.dart) | Inicialização `RustLib`, API gerada e serialização/deserialização Dart |
| [app/lib/src/rust/frb_generated.io.dart](../../../app/lib/src/rust/frb_generated.io.dart) | Integração específica de I/O/FFI da biblioteca nativa e ligação com o código gerado |

Esses arquivos podem ser lidos para entender a fronteira, mas não devem ser
corrigidos manualmente. A origem de uma mudança de contrato é `api.rs`; após
alterá-la, regenere com a versão combinada. Uma DLL antiga com bindings novos
pode falhar mesmo que cada lado, isoladamente, tenha compilado em outro momento.

## app/tool/verificar_ponte.dart

O [utilitário](../../../app/tool/verificar_ponte.dart) abre explicitamente
`../target/debug/chamada.dll`, por isso é executado a partir de `app/` após
compilar a biblioteca Debug. Confere a mensagem fixa, assina o stream e espera
`pronta` com prazo de cinco segundos.

Em seguida inicia uma chamada com endereço propositalmente inválido. Deve
receber `erro` e `encerrada`, sem abrir câmera ou rede. Chama encerrar,
solta a observação Rust, cancela a assinatura e descarta `RustLib`.
O teste exercita resultado simples, evento assíncrono e fechamento da ponte.

O histórico registra que o stream não terminava enquanto Rust mantinha o sink.
A função `encerrar_observacao` tornou essa liberação explícita. Esse utilitário
é uma forma pequena de verificar a correção sem confundir um problema de ponte
com uma falha de TURN.

## Onde a DLL entra no aplicativo

[app/windows/rust.cmake](../../../app/windows/rust.cmake) compila e copia a DLL
para o pacote. Sua leitura detalhada está em [infraestrutura](../infraestrutura/arquivos.md).
O `main()` Flutter chama `RustLib.init()` e verifica a ponte antes de habilitar
a interface. O [módulo Flutter](../flutter/README.md) acompanha a outra ponta.
