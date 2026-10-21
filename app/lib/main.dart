import 'package:flutter/material.dart';

void main() {
  runApp(const AplicativoChamada());
}

/// Interface inicial. A comunicação técnica será fornecida pela camada Rust.
class AplicativoChamada extends StatelessWidget {
  const AplicativoChamada({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'POC de chamada',
      debugShowCheckedModeBanner: false,
      home: Scaffold(
        appBar: AppBar(title: const Text('POC de chamada')),
        body: const Center(
          child: Padding(
            padding: EdgeInsets.all(24),
            // Texto estático para identificar a base, sem simular uma conexão.
            child: Text(
              'Base do projeto pronta.\n'
              'A chamada de áudio e vídeo ainda não foi implementada.',
              textAlign: TextAlign.center,
            ),
          ),
        ),
      ),
    );
  }
}
