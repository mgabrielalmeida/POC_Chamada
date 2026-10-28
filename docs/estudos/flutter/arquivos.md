# Flutter e Windows: leitura arquivo a arquivo

[Funcionamento geral](README.md) · [Ponte](../ponte-flutter-rust/README.md)

## app/lib/main.dart

[main.dart](../../../app/lib/main.dart) concentra a interface manual.
`TelaChamada` recebe apenas um possível erro de inicialização; seu `State`
possui controladores, assinatura, listener de ciclo de vida, imagens e flags.

| Método | Responsabilidade e motivo |
| --- | --- |
| `main` | Inicializa a DLL e verifica a ponte antes de construir a tela |
| `initState` | Assina eventos uma vez e registra a espera por limpeza na saída normal |
| `_receber` | Encaminha pixels à apresentação e atualiza estado/textos conforme o tipo do evento |
| `_apresentar` | Converte RGBA 320×240, evita conversões simultâneas por painel e descarta callbacks antigos |
| `_entrar` | Marca tentativa ativa, limpa a sessão anterior e encaminha servidor/sala a Rust |
| `_encerrar` | Bloqueia ações concorrentes enquanto aguarda a limpeza Rust |
| `_limparImagens` | Incrementa geração, zera caminho ICE e libera imagens |
| `_liberarCliente` | Encerra chamada, observação Rust e assinatura Dart nessa ordem |
| `dispose` | Libera controladores/listener e dispara a limpeza do cliente |
| `_video` | Monta painel com título, espera ou imagem, e espelha apenas quando solicitado |
| `build` | Compõe formulário, botões, mensagens e dois painéis |

Os valores iniciais `ws://127.0.0.1:8080` e `teste` ajudam no desenvolvimento
local. Em dois computadores, loopback apontaria para cada própria máquina;
o usuário precisa informar o servidor alcançável. A interface não deriva
automaticamente esse endereço do JSON de rede.

## Manifesto e configuração Flutter

| Arquivo | Função concreta |
| --- | --- |
| [pubspec.yaml](../../../app/pubspec.yaml) | Nome `poc_chamada`, versão, SDK Dart, Flutter, FRB 2.13.0 e lints. Não inclui plugin Flutter de WebRTC. |
| [pubspec.lock](../../../app/pubspec.lock) | Versões resolvidas das dependências Dart, preservadas para repetição |
| [analysis_options.yaml](../../../app/analysis_options.yaml) | Inclui as regras de análise do `flutter_lints` |
| [.metadata](../../../app/.metadata) | Metadados do projeto e migrações gerenciados pelas ferramentas Flutter |
| [.gitignore](../../../app/.gitignore) | Exclui caches e saídas próprias do projeto Flutter |
| [README.md](../../../app/README.md) | Entrada curta para execução e estudos da interface |

[flutter_rust_bridge.yaml](../../../app/flutter_rust_bridge.yaml) e cada arquivo
de `lib/src/rust/` são detalhados no [módulo da ponte](../ponte-flutter-rust/arquivos.md).

## Executor nativo: app/windows/runner

Flutter precisa de um executável Windows que crie a janela e hospede o engine.
Esses arquivos vêm do modelo do Flutter. Não contêm a negociação WebRTC da POC;
devem ser entendidos pelo papel que exercem na inicialização e no pacote.

| Arquivo | Funcionamento |
| --- | --- |
| [main.cpp](../../../app/windows/runner/main.cpp) | `wWinMain` associa console quando apropriado, inicializa COM, abre projeto Dart em `data`, passa argumentos, cria janela 1280×720 e executa o laço de mensagens Windows. |
| [flutter_window.h](../../../app/windows/runner/flutter_window.h) | Declara a janela que mantém projeto Dart e controlador Flutter. |
| [flutter_window.cpp](../../../app/windows/runner/flutter_window.cpp) | Cria `FlutterViewController`, registra plugins, encaixa a view, mostra após primeiro quadro e encaminha mensagens ao engine. Libera o controlador ao destruir. |
| [win32_window.h](../../../app/windows/runner/win32_window.h) | Declara a abstração de janela Win32, dimensões, conteúdo filho e eventos de criação/destruição. |
| [win32_window.cpp](../../../app/windows/runner/win32_window.cpp) | Registra classe de janela, cria/trata janela nativa, tamanho/DPI e integração visual com o sistema. |
| [utils.h](../../../app/windows/runner/utils.h) | Declara utilitários de console e argumentos. |
| [utils.cpp](../../../app/windows/runner/utils.cpp) | Liga stdout/stderr ao console e converte argumentos UTF-16 do Windows para UTF-8. |
| [resource.h](../../../app/windows/runner/resource.h) | Identificadores usados pelos recursos compilados. |
| [Runner.rc](../../../app/windows/runner/Runner.rc) | Define ícone e informações de versão do executável. |
| [runner.exe.manifest](../../../app/windows/runner/runner.exe.manifest) | Configura características Windows do processo, como DPI e compatibilidade. |
| [resources/app_icon.ico](../../../app/windows/runner/resources/app_icon.ico) | Ícone incorporado ao executável; não participa da chamada. |
| [CMakeLists.txt](../../../app/windows/runner/CMakeLists.txt) | Reúne fontes do runner, recursos, bibliotecas Flutter/dwmapi e dependência de `flutter_assemble`. |

## Arquivos de build e plugins

| Arquivo | Funcionamento |
| --- | --- |
| [windows/CMakeLists.txt](../../../app/windows/CMakeLists.txt) | Define binário, configurações Debug/Profile/Release, subdiretórios, regras de instalação e inclusão de `rust.cmake`. |
| [windows/rust.cmake](../../../app/windows/rust.cmake) | Integração autoral: compila Rust e instala DLL, helpers e licença; detalhada em infraestrutura. |
| [windows/flutter/CMakeLists.txt](../../../app/windows/flutter/CMakeLists.txt) | Regras mantidas pelo Flutter para engine, wrappers C++ e ferramenta de assemble. Usa conteúdo efêmero gerado. |
| [generated_plugin_registrant.h](../../../app/windows/flutter/generated_plugin_registrant.h) | Declara a função de registro automático dos plugins. |
| [generated_plugin_registrant.cc](../../../app/windows/flutter/generated_plugin_registrant.cc) | Implementa o registro gerado; não é lugar para adicionar manualmente uma chamada FRB. |
| [generated_plugins.cmake](../../../app/windows/flutter/generated_plugins.cmake) | Inclui bibliotecas/plugins gerados na compilação. |
| [windows/.gitignore](../../../app/windows/.gitignore) | Exclui arquivos efêmeros da plataforma. |

A compilação combina três trabalhos: engine/artefatos Dart pelo Flutter,
runner C++ pelo CMake e DLL pelo Cargo. Alterar `main.dart` não exige escrever
um codec em C++; alterar `api.rs` exige conferir bindings e a DLL correspondentes.
O [utilitário de ponte](../ponte-flutter-rust/arquivos.md) ajuda a isolar essa fronteira.
