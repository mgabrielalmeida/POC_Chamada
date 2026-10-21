//! Funções públicas que o gerador expõe ao Flutter.
use crate::frb_generated::StreamSink;
use anyhow::{Result, bail};
use std::sync::Mutex;
use tokio::{
    sync::{Mutex as MutexAsync, mpsc, watch},
    task::JoinHandle,
};

/// Os quadros são RGBA de 320×240. Eventos de estado têm `rgba` vazio.
#[derive(Clone, Debug)]
pub struct EventoChamada {
    pub tipo: String,
    pub texto: String,
    pub rgba: Vec<u8>,
}

struct Sessao {
    parar: watch::Sender<bool>,
    tarefa: JoinHandle<()>,
}

static OBSERVADOR: Mutex<Option<StreamSink<EventoChamada>>> = Mutex::new(None);
static SESSAO: MutexAsync<Option<Sessao>> = MutexAsync::const_new(None);

/// Há uma janela e um assinante por processo nesta POC.
pub fn observar_chamada(sink: StreamSink<EventoChamada>) {
    let _ = sink.add(EventoChamada {
        tipo: "pronta".into(),
        texto: "Pronto para criar ou entrar em uma sala.".into(),
        rgba: Vec::new(),
    });
    *OBSERVADOR.lock().unwrap() = Some(sink);
}

/// Solta o sink Rust antes de cancelar a assinatura Dart, fechando o stream.
pub fn encerrar_observacao() {
    OBSERVADOR.lock().unwrap().take();
}

/// A função apenas inicia a sessão; estados e vídeo retornam pelo stream.
pub async fn iniciar_chamada(endereco: String, sala: String) -> Result<()> {
    let mut sessao = SESSAO.lock().await;
    if sessao
        .as_ref()
        .is_some_and(|ativa| !ativa.tarefa.is_finished())
    {
        bail!("Já existe uma chamada neste aplicativo. Encerre antes de entrar novamente.");
    }
    if let Some(anterior) = sessao.take() {
        let _ = anterior.tarefa.await;
    }
    let (parar, parada) = watch::channel(false);
    let (eventos, mut recebidos) = mpsc::channel(8);
    let cancelamento = parar.clone();
    let tarefa = tokio::spawn(async move {
        let chamada = tokio::spawn(crate::chamada::executar(
            endereco, sala, false, eventos, parada,
        ));
        while let Some(evento) = recebidos.recv().await {
            let entregue = OBSERVADOR
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|sink| sink.add(evento).is_ok());
            if !entregue {
                let _ = cancelamento.send(true);
            }
        }
        let _ = chamada.await;
    });
    *sessao = Some(Sessao { parar, tarefa });
    Ok(())
}

/// Só retorna depois de fechar WebRTC e esperar os processos de mídia terminarem.
pub async fn encerrar_chamada() {
    let mut sessao = SESSAO.lock().await;
    if let Some(ativa) = sessao.take() {
        let _ = ativa.parar.send(true);
        let _ = ativa.tarefa.await;
    }
}

/// Confirma que a interface chamou a biblioteca nativa, sem abrir dispositivos.
pub fn verificar_ponte() -> String {
    "Ponte Flutter–Rust conectada.".to_owned()
}
