//! Confirma o primeiro quadro e os primeiros pacotes do microfone, depois libera.
use bytes::BytesMut;
use chamada::midia::{EventoMidia, Midia};
use rtc::shared::marshal::Unmarshal;
use tokio::{
    sync::mpsc,
    time::{Duration, timeout},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (envio, mut eventos) = mpsc::channel(8);
    let mut midia = Midia::iniciar(envio, false).await?;
    let audio = midia.audio.clone();
    let resultado = timeout(Duration::from_secs(15), async {
        let mut quadro = false;
        let mut som = false;
        let mut dados = [0; 1600];
        while !quadro || !som {
            tokio::select! {
                evento = eventos.recv() => match evento {
                    Some(EventoMidia::Quadro { local: true, rgba }) => {
                        println!("Quadro local decodificado: {} bytes RGBA.", rgba.len());
                        quadro = true;
                    }
                    Some(EventoMidia::Erro(erro)) => eprintln!("FFmpeg: {erro}"),
                    _ => {},
                },
                resultado = audio.recv_from(&mut dados), if !som => {
                    let (tamanho, _) = resultado?;
                    if let Ok(pacote) = rtc::rtp::packet::Packet::unmarshal(&mut BytesMut::from(&dados[..tamanho])) {
                        if pacote.header.payload_type == 111 && !pacote.payload.is_empty() {
                            println!("Áudio Opus recebido: {} bytes de payload RTP.", pacote.payload.len());
                            som = true;
                        }
                    }
                }
            }
            midia.verificar_processos()?;
        }
        Ok::<(), anyhow::Error>(())
    })
    .await;
    midia.encerrar().await;
    resultado??;
    println!("Captura verificada e recursos liberados. Não comprova qualidade ou chamada.");
    Ok(())
}
