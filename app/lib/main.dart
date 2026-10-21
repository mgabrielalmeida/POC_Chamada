import 'package:flutter/material.dart';

import 'src/rust/api.dart';
import 'src/rust/frb_generated.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  String mensagem;
  try {
    await RustLib.init();
    mensagem = await verificarPonte();
  } catch (erro) {
    mensagem = 'Não foi possível carregar Rust: $erro';
  }
  runApp(AplicativoChamada(mensagem: mensagem));
}

/// Interface inicial. A comunicação técnica será fornecida pela camada Rust.
class AplicativoChamada extends StatelessWidget {
  const AplicativoChamada({super.key, required this.mensagem});

  final String mensagem;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'POC de chamada',
      debugShowCheckedModeBanner: false,
      home: Scaffold(
        appBar: AppBar(title: const Text('POC de chamada')),
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            // O texto confirma uma chamada real à biblioteca nativa.
            child: Text(mensagem, textAlign: TextAlign.center),
          ),
        ),
      ),
    );
  }
}
