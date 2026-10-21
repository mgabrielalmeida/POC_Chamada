# Guia didático do código

## Por onde começar

Leia o README para executar a base. Depois abra `app/lib/main.dart`,
`rust/src/lib.rs` e `servidor/src/main.rs`, nessa ordem. Esses são os três pontos
de entrada; não existem camadas intermediárias próprias nesta versão.

## O aplicativo visual

`main()` em Dart inicia `AplicativoChamada`. O widget monta uma janela com título
e mensagem estática. Não abre câmera, microfone, socket ou conexão WebRTC.

Quando a ponte existir, os controles chamarão funções expostas por Rust e a tela
apresentará os estados recebidos. Flutter pode cuidar de detalhes visuais, como
campos de texto; decidir como conectar ou encerrar recursos cabe a Rust.

`app/windows/` é o executor nativo gerado pelo Flutter. Ele cria a janela e inicia
o mecanismo Flutter. Preserve os contratos desse código e evite colocar nele a
lógica de mídia ou de rede.

## A biblioteca técnica

`rust/src/lib.rs` ainda contém somente a descrição de sua responsabilidade.
`rust/Cargo.toml` permite gerar uma biblioteca Rust comum e uma biblioteca dinâmica
para a futura integração com a ponte. Isso, sozinho, não conecta Dart a Rust.

A implementação deverá ganhar módulos pequenos conforme a necessidade:
API para Flutter, captura e reprodução, conexão WebRTC e sinalização. Não existem
ainda funções, módulos ou bindings gerados para esses fluxos.

## O servidor

`servidor/src/main.rs` imprime uma mensagem e termina. Posteriormente ele receberá
conexões e encaminhará mensagens de uma pessoa para a outra na mesma sala.
Ele não deve executar a interface Flutter nem assumir o trabalho de mídia dos
clientes. Não há porta aberta ou servidor WebSocket implementado na base.

## A raiz Rust

`Cargo.toml` reúne os dois pacotes no mesmo workspace. `cargo check --workspace`
verifica ambos; `cargo run -p sinalizacao` executa somente o servidor.
`Cargo.lock` registra as versões resolvidas. As dependências técnicas serão
adicionadas quando usadas, mantendo o projeto inicial pequeno.

## Como continuar documentando

Ao integrar uma função real, explique onde ela é chamada, quais dados recebe,
qual resultado devolve e quem libera os recursos que ela abre. Comente decisões,
responsabilidades e trechos difíceis, sem repetir cada linha em linguagem natural.
Separe arquivos escritos manualmente dos gerados pela ponte e registre o comando
de geração antes de pedir que outra pessoa reproduza o procedimento.
