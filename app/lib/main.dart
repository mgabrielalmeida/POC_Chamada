import 'dart:async';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';

import 'src/rust/api.dart';
import 'src/rust/frb_generated.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  String? erro;
  try {
    await RustLib.init();
    await verificarPonte();
  } catch (falha) {
    erro = 'Não foi possível carregar Rust: $falha';
  }
  runApp(
    MaterialApp(
      title: 'POC de chamada',
      debugShowCheckedModeBanner: false,
      home: TelaChamada(erroInicial: erro),
    ),
  );
}

/// Só apresenta estados e imagens. Captura, áudio e protocolos pertencem a Rust.
class TelaChamada extends StatefulWidget {
  const TelaChamada({super.key, this.erroInicial});
  final String? erroInicial;

  @override
  State<TelaChamada> createState() => _TelaChamadaState();
}

class _TelaChamadaState extends State<TelaChamada> {
  final _servidor = TextEditingController(text: 'ws://127.0.0.1:8080');
  final _sala = TextEditingController(text: 'teste');
  StreamSubscription<EventoChamada>? _assinatura;
  AppLifecycleListener? _ciclo;
  String _estado = 'Carregando Rust…';
  String? _caminhoIce;
  String? _erro;
  bool _pronta = false;
  bool _ativa = false;
  bool _encerrando = false;
  bool _decodificandoLocal = false;
  bool _decodificandoRemoto = false;
  int _geracao = 0;
  ui.Image? _local;
  ui.Image? _remoto;

  @override
  void initState() {
    super.initState();
    _erro = widget.erroInicial;
    if (_erro != null) return;
    _assinatura = observarChamada().listen(
      _receber,
      onError: (Object erro) {
        if (mounted) setState(() => _erro = erro.toString());
      },
    );
    // A saída normal espera Rust liberar os dispositivos antes de fechar a janela.
    _ciclo = AppLifecycleListener(
      onExitRequested: () async {
        await encerrarChamada();
        return ui.AppExitResponse.exit;
      },
    );
  }

  void _receber(EventoChamada evento) {
    if (!mounted) return;
    if (evento.tipo == 'video_local' || evento.tipo == 'video_remoto') {
      _apresentar(evento.rgba, evento.tipo == 'video_local');
      return;
    }
    setState(() {
      switch (evento.tipo) {
        case 'pronta':
          _pronta = true;
          _estado = evento.texto;
        case 'estado':
          _estado = evento.texto;
        case 'caminho_ice':
          _caminhoIce = evento.texto;
        case 'erro':
          _erro = evento.texto;
        case 'aviso':
          _erro = 'Mídia: ${evento.texto}';
        case 'encerrada':
          _ativa = false;
          _estado = evento.texto;
          _limparImagens();
      }
    });
  }

  void _apresentar(Uint8List rgba, bool local) {
    if (!_ativa || (local ? _decodificandoLocal : _decodificandoRemoto)) return;
    if (local) {
      _decodificandoLocal = true;
    } else {
      _decodificandoRemoto = true;
    }
    final geracao = _geracao;
    // Converter RGBA em imagem é apresentação visual, sem codec de mídia em Dart.
    ui.decodeImageFromPixels(rgba, 320, 240, ui.PixelFormat.rgba8888, (imagem) {
      if (local) {
        _decodificandoLocal = false;
      } else {
        _decodificandoRemoto = false;
      }
      if (!mounted || geracao != _geracao || !_ativa) {
        imagem.dispose();
        return;
      }
      setState(() {
        if (local) {
          _local?.dispose();
          _local = imagem;
        } else {
          _remoto?.dispose();
          _remoto = imagem;
        }
      });
    });
  }

  Future<void> _entrar() async {
    setState(() {
      _ativa = true;
      _erro = null;
      _estado = 'Conectando à sinalização…';
      _limparImagens();
    });
    try {
      await iniciarChamada(
        endereco: _servidor.text.trim(),
        sala: _sala.text.trim(),
      );
    } catch (erro) {
      if (mounted) {
        setState(() {
          _ativa = false;
          _erro = erro.toString();
        });
      }
    }
  }

  Future<void> _encerrar() async {
    setState(() => _encerrando = true);
    await encerrarChamada();
    if (mounted) setState(() => _encerrando = false);
  }

  void _limparImagens() {
    _geracao++;
    _caminhoIce = null;
    _local?.dispose();
    _remoto?.dispose();
    _local = null;
    _remoto = null;
  }

  @override
  void dispose() {
    _ciclo?.dispose();
    if (widget.erroInicial == null) {
      unawaited(_liberarCliente());
    }
    _servidor.dispose();
    _sala.dispose();
    _limparImagens();
    super.dispose();
  }

  Future<void> _liberarCliente() async {
    await encerrarChamada();
    await encerrarObservacao();
    await _assinatura?.cancel();
  }

  Widget _video(String titulo, ui.Image? imagem) => Expanded(
    child: Column(
      children: [
        Text(titulo),
        const SizedBox(height: 8),
        Expanded(
          child: Container(
            width: double.infinity,
            color: Colors.black,
            child: imagem == null
                ? const Center(
                    child: Text(
                      'Aguardando vídeo',
                      style: TextStyle(color: Colors.white),
                    ),
                  )
                : RawImage(image: imagem, fit: BoxFit.contain),
          ),
        ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: const Text('POC de chamada 1:1')),
    body: Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                flex: 2,
                child: TextField(
                  controller: _servidor,
                  enabled: !_ativa && !_encerrando,
                  decoration: const InputDecoration(
                    labelText: 'Servidor de sinalização',
                  ),
                ),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: TextField(
                  controller: _sala,
                  enabled: !_ativa && !_encerrando,
                  decoration: const InputDecoration(labelText: 'Sala'),
                ),
              ),
              const SizedBox(width: 16),
              FilledButton(
                onPressed: _pronta && !_ativa && !_encerrando ? _entrar : null,
                child: const Text('Criar / entrar'),
              ),
              const SizedBox(width: 8),
              OutlinedButton(
                onPressed: _ativa && !_encerrando ? _encerrar : null,
                child: const Text('Encerrar'),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Text(_estado),
          if (_caminhoIce != null) Text(_caminhoIce!),
          if (_erro != null)
            Text(_erro!, style: const TextStyle(color: Colors.red)),
          const SizedBox(height: 16),
          Expanded(
            child: Row(
              children: [
                _video('Seu vídeo', _local),
                const SizedBox(width: 16),
                _video('Outra pessoa', _remoto),
              ],
            ),
          ),
        ],
      ),
    ),
  );
}
