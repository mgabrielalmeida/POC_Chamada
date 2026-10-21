//! Uma conexão WebRTC por sessão. A sinalização só transporta a negociação SDP.
use crate::api::EventoChamada as Evento;
use crate::midia::{EventoMidia, Midia};
use anyhow::{Context, Result, bail};
use bytes::BytesMut;
use futures_util::{SinkExt, StreamExt};
use rtc::{
    media_stream::MediaStreamTrack,
    rtp,
    rtp_transceiver::rtp_sender::{
        RTCRtpCodec, RTCRtpCodecParameters, RTCRtpCodingParameters, RTCRtpEncodingParameters,
        RtpCodecKind,
    },
    shared::marshal::{Marshal, Unmarshal},
};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, watch},
    task::JoinSet,
    time::timeout,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use webrtc::{
    media_stream::{
        track_local::{TrackLocal, static_rtp::TrackLocalStaticRTP},
        track_remote::{TrackRemote, TrackRemoteEvent},
    },
    peer_connection::{
        MediaEngine, PeerConnection, PeerConnectionBuilder, PeerConnectionEventHandler,
        RTCConfigurationBuilder, RTCIceGatheringState, RTCPeerConnectionState,
        RTCSessionDescription, Registry, register_default_interceptors,
    },
};

impl Evento {
    pub(crate) fn texto(tipo: &str, texto: impl Into<String>) -> Self {
        Self {
            tipo: tipo.into(),
            texto: texto.into(),
            rgba: Vec::new(),
        }
    }
}

enum Mudanca {
    Estado(RTCPeerConnectionState),
    Trilha(Arc<dyn TrackRemote>),
}

struct Observador {
    coleta: watch::Sender<bool>,
    mudancas: mpsc::UnboundedSender<Mudanca>,
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for Observador {
    async fn on_ice_gathering_state_change(&self, estado: RTCIceGatheringState) {
        if estado == RTCIceGatheringState::Complete {
            let _ = self.coleta.send(true);
        }
    }
    async fn on_connection_state_change(&self, estado: RTCPeerConnectionState) {
        let _ = self.mudancas.send(Mudanca::Estado(estado));
    }
    async fn on_track(&self, trilha: Arc<dyn TrackRemote>) {
        // O callback retorna imediatamente; a leitura é feita numa tarefa da sessão.
        let _ = self.mudancas.send(Mudanca::Trilha(trilha));
    }
}

#[derive(Default)]
struct Recursos {
    sinalizacao: Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>,
    conexao: Option<Arc<dyn PeerConnection>>,
    midia: Option<Midia>,
    tarefas: JoinSet<()>,
}

/// Todos os caminhos, incluindo erro e cancelamento, fecham os mesmos recursos.
pub async fn executar(
    endereco: String,
    sala: String,
    sintetica: bool,
    eventos: mpsc::Sender<Evento>,
    mut parar: watch::Receiver<bool>,
) -> Result<()> {
    let mut recursos = Recursos::default();
    let resultado = tokio::select! {
        _ = parar.changed() => Ok(()),
        resultado = conectar(&endereco, &sala, sintetica, &eventos, &mut recursos) => resultado,
    };
    if let Some(ws) = &mut recursos.sinalizacao {
        // A desconexão normal avisa o servidor antes de liberar o socket.
        let _ = timeout(Duration::from_secs(1), ws.close(None)).await;
    }
    if let Some(conexao) = &recursos.conexao {
        let _ = conexao.close().await;
    }
    recursos.tarefas.abort_all();
    while recursos.tarefas.join_next().await.is_some() {}
    if let Some(midia) = &mut recursos.midia {
        midia.encerrar().await;
    }
    if let Err(erro) = &resultado {
        let _ = eventos
            .send(Evento::texto("erro", format!("{erro:#}")))
            .await;
    }
    let _ = eventos
        .send(Evento::texto(
            "encerrada",
            "Chamada encerrada. Dispositivos liberados.",
        ))
        .await;
    resultado
}

async fn conectar(
    endereco: &str,
    sala: &str,
    sintetica: bool,
    eventos: &mpsc::Sender<Evento>,
    recursos: &mut Recursos,
) -> Result<()> {
    if !endereco.starts_with("ws://") {
        bail!("Na Etapa 1 use ws://IP_DO_SERVIDOR:8080");
    }
    let (ws, _) = timeout(Duration::from_secs(10), connect_async(endereco))
        .await
        .context("Tempo esgotado ao conectar à sinalização")??;
    recursos.sinalizacao = Some(ws);
    let ws = recursos.sinalizacao.as_mut().unwrap();
    ws.send(Message::Text(
        json!({"tipo":"entrar","sala":sala}).to_string().into(),
    ))
    .await?;
    let ofertante = loop {
        let mensagem = ws.next().await.context("Servidor encerrou a conexão")??;
        if let Message::Ping(dados) = mensagem {
            ws.send(Message::Pong(dados)).await?;
            continue;
        }
        if !mensagem.is_text() {
            continue;
        }
        let mensagem: Value = serde_json::from_str(mensagem.to_text()?)?;
        match mensagem["tipo"].as_str().unwrap_or("") {
            "aguardando" => {
                eventos
                    .send(Evento::texto(
                        "estado",
                        "Aguardando a outra pessoa na sala.",
                    ))
                    .await?;
            }
            "pronto" => break mensagem["ofertante"].as_bool().context("Papel inválido")?,
            "erro" => bail!(
                "{}",
                mensagem["mensagem"]
                    .as_str()
                    .unwrap_or("Erro de sinalização")
            ),
            _ => bail!("Resposta inesperada do servidor"),
        }
    };
    eventos
        .send(Evento::texto(
            "estado",
            "Preparando câmera, microfone e WebRTC…",
        ))
        .await?;
    let (envio_midia, mut eventos_midia) = mpsc::channel(8);
    recursos.midia = Some(Midia::iniciar(envio_midia, sintetica).await?);
    let midia = recursos.midia.as_ref().unwrap();
    let destino_video = midia.destino_video;
    let destino_audio = midia.destino_audio;
    let captura_video = midia.video.clone();
    let captura_audio = midia.audio.clone();
    let (coleta, mut coleta_rx) = watch::channel(false);
    let (mudancas, mut mudancas_rx) = mpsc::unbounded_channel();
    let mut motor = MediaEngine::default();
    let mut trilhas = Vec::new();
    for (nome, mime, frequencia, canais, payload, tipo, socket) in [
        (
            "video",
            "video/VP8",
            90000,
            0,
            96,
            RtpCodecKind::Video,
            captura_video,
        ),
        (
            "audio",
            "audio/opus",
            48000,
            2,
            111,
            RtpCodecKind::Audio,
            captura_audio,
        ),
    ] {
        let codec = RTCRtpCodec {
            mime_type: mime.into(),
            clock_rate: frequencia,
            channels: canais,
            ..Default::default()
        };
        motor.register_codec(
            RTCRtpCodecParameters {
                rtp_codec: codec.clone(),
                payload_type: payload,
                ..Default::default()
            },
            tipo,
        )?;
        let ssrc = rand::random::<u32>();
        let trilha = Arc::new(TrackLocalStaticRTP::new(MediaStreamTrack::new(
            "chamada".into(),
            nome.into(),
            nome.into(),
            tipo,
            vec![RTCRtpEncodingParameters {
                rtp_coding_parameters: RTCRtpCodingParameters {
                    ssrc: Some(ssrc),
                    ..Default::default()
                },
                codec,
                ..Default::default()
            }],
        )));
        trilhas.push((trilha, socket, ssrc));
    }
    let interceptores = register_default_interceptors(Registry::new(), &mut motor)?;
    let conexao: Arc<dyn PeerConnection> = Arc::new(
        PeerConnectionBuilder::new()
            .with_configuration(RTCConfigurationBuilder::default().build())
            .with_media_engine(motor)
            .with_interceptor_registry(interceptores)
            .with_handler(Arc::new(Observador { coleta, mudancas }))
            // Sem STUN/TURN externo na LAN. O wildcard reúne endereços locais.
            .with_udp_addrs(vec!["0.0.0.0:0"])
            .build()
            .await?,
    );
    recursos.conexao = Some(conexao.clone());
    for (trilha, _, _) in &trilhas {
        conexao
            .add_track(trilha.clone() as Arc<dyn TrackLocal>)
            .await?;
    }
    if ofertante {
        conexao
            .set_local_description(conexao.create_offer(None).await?)
            .await?;
        aguardar_coleta(&mut coleta_rx).await?;
        let descricao = conexao
            .local_description()
            .await
            .context("Oferta local ausente")?;
        ws.send(Message::Text(
            json!({"tipo":"descricao","descricao":descricao})
                .to_string()
                .into(),
        ))
        .await?;
    }
    let mut negociada = false;
    let mut conectada = false;
    let mut vigilancia = tokio::time::interval(Duration::from_secs(1));
    let inicio = tokio::time::Instant::now();
    loop {
        tokio::select! {
            mensagem = ws.next() => {
                let mensagem = mensagem.context("Sinalização desconectada")??;
                match mensagem {
                    Message::Ping(dados) => ws.send(Message::Pong(dados)).await?,
                    Message::Text(texto) => {
                        let mensagem: Value = serde_json::from_str(&texto)?;
                        match mensagem["tipo"].as_str().unwrap_or("") {
                            "descricao" => {
                                if negociada { bail!("Negociação repetida não é suportada nesta POC"); }
                                let descricao: RTCSessionDescription = serde_json::from_value(mensagem["descricao"].clone())?;
                                conexao.set_remote_description(descricao).await?;
                                if !ofertante {
                                    conexao.set_local_description(conexao.create_answer(None).await?).await?;
                                    aguardar_coleta(&mut coleta_rx).await?;
                                    let descricao = conexao.local_description().await.context("Resposta local ausente")?;
                                    ws.send(Message::Text(json!({"tipo":"descricao","descricao":descricao}).to_string().into())).await?;
                                }
                                negociada = true;
                            }
                            "saiu" => return Ok(()),
                            "erro" => bail!("Erro de sinalização: {}", mensagem["mensagem"]),
                            _ => bail!("Mensagem inesperada durante a chamada"),
                        }
                    }
                    Message::Close(_) => return Ok(()),
                    _ => {},
                }
            }
            Some(mudanca) = mudancas_rx.recv() => match mudanca {
                Mudanca::Estado(estado) => {
                    if estado == RTCPeerConnectionState::Connected && !conectada {
                        conectada = true;
                        eventos.send(Evento::texto("estado", "WebRTC conectado. Áudio e vídeo em transmissão.")).await?;
                        for (trilha, socket, ssrc) in trilhas.drain(..) {
                            recursos.tarefas.spawn(enviar_rtp(socket, trilha, ssrc));
                        }
                    } else if matches!(estado, RTCPeerConnectionState::Failed | RTCPeerConnectionState::Closed | RTCPeerConnectionState::Disconnected) {
                        bail!("Conexão WebRTC encerrada: {estado}");
                    }
                }
                Mudanca::Trilha(trilha) => {
                    let video = trilha.kind().await == RtpCodecKind::Video;
                    recursos.tarefas.spawn(receber_rtp(trilha, if video { destino_video } else { destino_audio }, if video { 96 } else { 111 }));
                }
            },
            Some(evento) = eventos_midia.recv() => match evento {
                EventoMidia::Quadro { local, rgba } => {
                    // O canal é limitado; quadros são descartados quando a UI atrasa.
                    let _ = eventos.try_send(Evento { tipo: if local { "video_local" } else { "video_remoto" }.into(), texto: String::new(), rgba });
                }
                EventoMidia::AudioDecodificado => { let _ = eventos.try_send(Evento::texto("audio_teste", "PCM remoto decodificado")); }
                EventoMidia::Erro(erro) => {
                    eprintln!("Mídia: {erro}");
                    let _ = eventos.try_send(Evento::texto("aviso", erro));
                }
            },
            _ = vigilancia.tick() => {
                recursos.midia.as_mut().unwrap().verificar_processos()?;
                if !conectada && inicio.elapsed() > Duration::from_secs(30) { bail!("A conexão WebRTC não foi estabelecida em 30 segundos"); }
                if eventos.is_closed() { return Ok(()); }
            }
        }
    }
}

async fn aguardar_coleta(coleta: &mut watch::Receiver<bool>) -> Result<()> {
    timeout(
        Duration::from_secs(10),
        coleta.wait_for(|completa| *completa),
    )
    .await
    .context("Tempo esgotado ao reunir os endereços ICE")??;
    Ok(())
}

async fn enviar_rtp(socket: Arc<UdpSocket>, trilha: Arc<TrackLocalStaticRTP>, ssrc: u32) {
    let mut dados = vec![0; 1600];
    while let Ok((tamanho, _)) = socket.recv_from(&mut dados).await {
        if let Ok(mut pacote) =
            rtp::packet::Packet::unmarshal(&mut BytesMut::from(&dados[..tamanho]))
        {
            // RTCP local não é áudio/vídeo e não deve entrar numa trilha RTP.
            if !matches!(pacote.header.payload_type, 96 | 111) {
                continue;
            }
            pacote.header.ssrc = ssrc;
            if trilha.write_rtp(pacote).await.is_err() {
                break;
            }
        }
    }
}

async fn receber_rtp(trilha: Arc<dyn TrackRemote>, destino: std::net::SocketAddr, payload: u8) {
    let Ok(socket) = UdpSocket::bind("127.0.0.1:0").await else {
        return;
    };
    while let Some(evento) = trilha.poll().await {
        if let TrackRemoteEvent::OnRtpPacket(mut pacote) = evento {
            pacote.header.payload_type = payload;
            // O decodificador local não negocia extensões de cabeçalho WebRTC.
            pacote.header.extension = false;
            pacote.header.extensions.clear();
            if let Ok(dados) = pacote.marshal() {
                if socket.send_to(&dados, destino).await.is_err() {
                    break;
                }
            }
        }
    }
}
