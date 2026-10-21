// Execute `dart run tool/verificar_ponte.dart` após `cargo build -p chamada`.
import 'dart:io';

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
  RustLib.dispose();
}
