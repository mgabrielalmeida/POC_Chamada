// Execute `dart run tool/verificar_ponte.dart` após `cargo build -p chamada`.
import 'dart:io';
import 'dart:async';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:poc_chamada/src/rust/api.dart';
import 'package:poc_chamada/src/rust/frb_generated.dart';

Future<void> main() async {
  final biblioteca = File('../target/debug/chamada.dll').absolute.path;
  await RustLib.init(externalLibrary: ExternalLibrary.open(biblioteca));
  final resposta = await verificarPonte();
  if (resposta != 'Ponte Flutter–Rust conectada.') {
    throw StateError('Resposta inesperada da biblioteca: $resposta');
  }
  stdout.writeln(resposta);
  // Verifica também stream, erro assíncrono e cancelamento sem abrir dispositivos.
  final pronta = Completer<void>();
  final encerrada = Completer<void>();
  var recebeuErro = false;
  final assinatura = observarChamada().listen((evento) {
    if (evento.tipo == 'pronta' && !pronta.isCompleted) pronta.complete();
    if (evento.tipo == 'erro') recebeuErro = true;
    if (evento.tipo == 'encerrada' && !encerrada.isCompleted) {
      encerrada.complete();
    }
  });
  await pronta.future.timeout(const Duration(seconds: 5));
  await iniciarChamada(endereco: 'endereco-invalido', sala: 'teste');
  await encerrada.future.timeout(const Duration(seconds: 5));
  await encerrarChamada();
  await encerrarObservacao();
  await assinatura.cancel().timeout(const Duration(seconds: 5));
  if (!recebeuErro) throw StateError('O erro Rust não chegou ao stream Dart.');
  stdout.writeln('Stream, erro assíncrono e encerramento verificados.');
  RustLib.dispose();
}
