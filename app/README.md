# Aplicativo Windows

Interface visual da POC. Instruções de execução estão no [README principal](../README.md).

`lib/main.dart` contém a interface e consome a API Rust via Flutter-Rust Bridge.
O [guia da Etapa 1](../docs/desenvolvimento/etapa-1-lan.md) explica preparação, sala e validação.
Para estudar, leia [Flutter](../docs/estudos/flutter/README.md) e
[ponte Flutter–Rust](../docs/estudos/ponte-flutter-rust/README.md), com explicações
do funcionamento e de cada arquivo, incluindo o executor Windows.
`windows/` contém o executor nativo gerado pelo Flutter; mantenha os nomes e
contratos exigidos pelas ferramentas. Seus comentários originais foram preservados.
