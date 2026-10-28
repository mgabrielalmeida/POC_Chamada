//! Transporte UDP/ICE/DTLS/SRTP real em IPv6 local, sem dispositivos ou FFmpeg.
use bytes::Bytes;
use rtc::{
    media_stream::MediaStreamTrack,
    peer_connection::transport::RTCIceCandidateType,
    rtp::{header::Header, packet::Packet},
    rtp_transceiver::rtp_sender::{
        RTCRtpCodec, RTCRtpCodecParameters, RTCRtpCodingParameters, RTCRtpEncodingParameters,
        RtpCodecKind,
    },
};
use std::{net::IpAddr, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc, watch},
    time::{interval, timeout},
};
use webrtc::{
    media_stream::{
        Track,
        track_local::{TrackLocal, static_rtp::TrackLocalStaticRTP},
        track_remote::{TrackRemote, TrackRemoteEvent},
    },
    peer_connection::{
        MediaEngine, PeerConnection, PeerConnectionBuilder, PeerConnectionEventHandler,
        RTCConfiguration, RTCConfigurationBuilder, RTCIceGatheringState, RTCIceServer,
        RTCIceTransportPolicy,
    },
};

struct Observador {
    coleta: watch::Sender<bool>,
    recebidos: mpsc::Sender<(RtpCodecKind, Vec<u8>)>,
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for Observador {
    async fn on_ice_gathering_state_change(&self, estado: RTCIceGatheringState) {
        if estado == RTCIceGatheringState::Complete {
            let _ = self.coleta.send(true);
        }
    }
    async fn on_track(&self, trilha: Arc<dyn TrackRemote>) {
        let recebidos = self.recebidos.clone();
        tokio::spawn(async move {
            let tipo = trilha.kind().await;
            while let Some(evento) = trilha.poll().await {
                if let TrackRemoteEvent::OnRtpPacket(pacote) = evento
                    && recebidos
                        .send((tipo, pacote.payload.to_vec()))
                        .await
                        .is_err()
                {
                    break;
                }
            }
        });
    }
}

async fn criar(
    config: RTCConfiguration,
    endereco_local: &str,
) -> (
    Arc<dyn PeerConnection>,
    Vec<Arc<TrackLocalStaticRTP>>,
    watch::Receiver<bool>,
    mpsc::Receiver<(RtpCodecKind, Vec<u8>)>,
) {
    let (coleta, pronta) = watch::channel(false);
    let (recebidos, pacotes) = mpsc::channel(64);
    let mut motor = MediaEngine::default();
    let mut trilhas = Vec::new();
    for (nome, mime, frequencia, canais, payload, tipo) in [
        ("video", "video/VP8", 90000, 0, 96, RtpCodecKind::Video),
        ("audio", "audio/opus", 48000, 2, 111, RtpCodecKind::Audio),
    ] {
        let codec = RTCRtpCodec {
            mime_type: mime.into(),
            clock_rate: frequencia,
            channels: canais,
            ..Default::default()
        };
        motor
            .register_codec(
                RTCRtpCodecParameters {
                    rtp_codec: codec.clone(),
                    payload_type: payload,
                },
                tipo,
            )
            .unwrap();
        trilhas.push(Arc::new(TrackLocalStaticRTP::new(MediaStreamTrack::new(
            "ipv6".into(),
            nome.into(),
            nome.into(),
            tipo,
            vec![RTCRtpEncodingParameters {
                rtp_coding_parameters: RTCRtpCodingParameters {
                    ssrc: Some(rand::random()),
                    ..Default::default()
                },
                codec,
                ..Default::default()
            }],
        ))));
    }
    let pc: Arc<dyn PeerConnection> = Arc::new(
        PeerConnectionBuilder::new()
            .with_configuration(config)
            .with_media_engine(motor)
            .with_handler(Arc::new(Observador { coleta, recebidos }))
            .with_udp_addrs(vec![endereco_local.to_owned()])
            .build()
            .await
            .unwrap(),
    );
    for trilha in &trilhas {
        pc.add_track(trilha.clone()).await.unwrap();
    }
    (pc, trilhas, pronta, pacotes)
}

async fn ensaiar(config: RTCConfiguration, relay: bool, endereco_local: &str) {
    let (a, trilhas_a, mut coleta_a, mut pacotes_a) = criar(config.clone(), endereco_local).await;
    let (b, trilhas_b, mut coleta_b, mut pacotes_b) = criar(config, endereco_local).await;
    // A limpeza também ocorre quando a negociação ou o transporte falha.
    let resultado = timeout(Duration::from_secs(20), async {
        a.set_local_description(a.create_offer(None).await.unwrap()).await.unwrap();
        coleta_a.wait_for(|pronta| *pronta).await.unwrap();
        let oferta = a.local_description().await.unwrap();
        if !relay { assert!(oferta.sdp.contains("::1")); }
        b.set_remote_description(oferta).await.unwrap();
        b.set_local_description(b.create_answer(None).await.unwrap()).await.unwrap();
        coleta_b.wait_for(|pronta| *pronta).await.unwrap();
        a.set_remote_description(b.local_description().await.unwrap()).await.unwrap();
        let mut vistos = [[false; 2]; 2];
        let mut pulso = interval(Duration::from_millis(20));
        let mut sequencia: u16 = 0;
        while vistos.iter().flatten().any(|visto| !visto) {
            tokio::select! {
                _ = pulso.tick() => {
                    sequencia = sequencia.wrapping_add(1);
                    for trilhas in [&trilhas_a, &trilhas_b] {
                        for (i, trilha) in trilhas.iter().enumerate() {
                            trilha.write_rtp(Packet {
                                header: Header { version: 2, payload_type: if i == 0 { 96 } else { 111 }, sequence_number: sequencia, timestamp: u32::from(sequencia) * 960, ssrc: trilha.ssrcs().await[0], ..Default::default() },
                                payload: Bytes::from_static(b"payload-ipv6"),
                            }).await.unwrap();
                        }
                    }
                }
                Some((tipo, dados)) = pacotes_a.recv() => {
                    assert_eq!(dados, b"payload-ipv6");
                    vistos[0][usize::from(tipo == RtpCodecKind::Audio)] = true;
                }
                Some((tipo, dados)) = pacotes_b.recv() => {
                    assert_eq!(dados, b"payload-ipv6");
                    vistos[1][usize::from(tipo == RtpCodecKind::Audio)] = true;
                }
            }
        }
        for pc in [&a, &b] {
            let remetente = pc.get_senders().await.remove(0);
            let transporte = remetente.transport().await.unwrap().unwrap();
            let par = transporte.ice_transport().get_selected_candidate_pair().await.unwrap().unwrap();
            for candidato in [par.local(), par.remote()] {
                assert!(candidato.address.parse::<IpAddr>().unwrap().is_ipv6());
                assert_eq!(candidato.typ, if relay { RTCIceCandidateType::Relay } else { RTCIceCandidateType::Host });
            }
            eprintln!("IPv6 selecionado: local={:?}; remoto={:?}", par.local(), par.remote());
        }
    }).await;
    a.close().await.unwrap();
    b.close().await.unwrap();
    assert!(
        resultado.is_ok(),
        "ICE/mídia IPv6 não completou em 20 segundos"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn rtp_de_audio_e_video_bidirecional_em_ipv6_com_repeticao() {
    for _ in 0..2 {
        ensaiar(RTCConfigurationBuilder::default().build(), false, "[::1]:0").await;
    }
}

#[tokio::test]
async fn runtime_ice_resolve_hostname_para_ipv6() {
    let runtime = webrtc::runtime::default_runtime().unwrap();
    let enderecos = runtime.resolve_host("localhost:3478").await.unwrap();
    assert!(enderecos.iter().any(|endereco| endereco.is_ipv6()));
}

// Coturn de teste separado do serviço público: listening-ip=::1, relay-ip=::1,
// allocation-default-address-family=ipv6, lt-cred-mech e allow-loopback-peers
// somente nesse serviço isolado de teste. Nunca habilitar no servidor público.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Requer coturn IPv6 local e CHAMADA_TESTE_TURN_USUARIO/SENHA/PORTA"]
async fn rtp_bidirecional_por_turn_ipv6_real() {
    let porta: u16 = std::env::var("CHAMADA_TESTE_TURN_PORTA")
        .unwrap()
        .parse()
        .unwrap();
    let config = RTCConfigurationBuilder::default()
        .with_ice_servers(vec![RTCIceServer {
            urls: vec![format!(
                "turn:{}:{porta}?transport=udp",
                std::env::var("CHAMADA_TESTE_TURN_HOST").unwrap_or_else(|_| "localhost".into())
            )],
            username: std::env::var("CHAMADA_TESTE_TURN_USUARIO").unwrap(),
            credential: std::env::var("CHAMADA_TESTE_TURN_SENHA").unwrap(),
        }])
        .with_ice_transport_policy(RTCIceTransportPolicy::Relay)
        .build();
    let endereco_local =
        std::env::var("CHAMADA_TESTE_TURN_BIND").unwrap_or_else(|_| "[::1]:0".into());
    ensaiar(config, true, &endereco_local).await;
}

/// Resposta STUN controlada: comprova resolução/UDP/serialização srflx, não alcance público.
#[tokio::test]
async fn stun_por_hostname_coleta_candidato_srflx_ipv6() {
    use rtc::stun::{
        message::{BINDING_SUCCESS, Message},
        xoraddr::XorMappedAddress,
    };
    let socket = tokio::net::UdpSocket::bind("[::1]:0").await.unwrap();
    let porta = socket.local_addr().unwrap().port();
    let servidor = tokio::spawn(async move {
        let mut dados = [0; 2048];
        let (n, origem) = socket.recv_from(&mut dados).await.unwrap();
        assert!(origem.is_ipv6());
        let mut pedido = Message::new();
        pedido.raw = dados[..n].to_vec();
        pedido.decode().unwrap();
        let mut resposta = Message::new();
        resposta
            .build(&[
                Box::new(pedido.transaction_id),
                Box::new(BINDING_SUCCESS),
                Box::new(XorMappedAddress {
                    ip: "2001:db8::20".parse().unwrap(),
                    port: origem.port(),
                }),
            ])
            .unwrap();
        socket.send_to(&resposta.raw, origem).await.unwrap();
    });
    let config = RTCConfigurationBuilder::default()
        .with_ice_servers(vec![RTCIceServer {
            urls: vec![format!("stun:localhost:{porta}")],
            ..Default::default()
        }])
        .build();
    let (pc, _trilhas, mut coleta, _pacotes) = criar(config, "[::1]:0").await;
    pc.set_local_description(pc.create_offer(None).await.unwrap())
        .await
        .unwrap();
    let resultado = timeout(Duration::from_secs(5), coleta.wait_for(|pronta| *pronta)).await;
    let descricao = pc.local_description().await.unwrap();
    pc.close().await.unwrap();
    servidor.abort();
    assert!(resultado.is_ok(), "Coleta STUN IPv6 não completou");
    assert!(
        descricao
            .sdp
            .lines()
            .any(|linha| linha.contains("2001:db8::20") && linha.contains("typ srflx")),
        "Sem srflx IPv6 no SDP"
    );
}
