//! Executa o transporte real com vídeo e áudio gerados, sem câmera ou alto-falante.
use chamada::{api::EventoChamada, chamada::executar};
use tokio::{
    net::TcpListener,
    sync::{mpsc, watch},
    time::{Duration, timeout},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Requer scripts/preparar-midia.ps1; inicia processos FFmpeg locais"]
async fn duas_chamadas_seguidas_transportam_e_decodificam_video_e_audio() {
    // O mesmo ensaio serve ao modo normal e ao TURN forçado do pacote.
    // Lê somente a política; nunca imprime o JSON que contém credenciais.
    let somente_turn = std::env::var_os("CHAMADA_CONFIG")
        .and_then(|caminho| std::fs::read_to_string(caminho).ok())
        .and_then(|texto| serde_json::from_str::<serde_json::Value>(&texto).ok())
        .and_then(|json| json["somente_retransmissao"].as_bool())
        .unwrap_or(false);
    let ouvinte = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endereco = format!("ws://{}", ouvinte.local_addr().unwrap());
    let servidor = tokio::spawn(sinalizacao::executar(ouvinte));
    for rodada in 1..=2 {
        let (eventos_a, mut a) = mpsc::channel::<EventoChamada>(32);
        let (eventos_b, mut b) = mpsc::channel::<EventoChamada>(32);
        let (parar_a, parada_a) = watch::channel(false);
        let (parar_b, parada_b) = watch::channel(false);
        let tarefa_a = tokio::spawn(executar(
            endereco.clone(),
            "teste".into(),
            true,
            eventos_a,
            parada_a,
        ));
        let tarefa_b = tokio::spawn(executar(
            endereco.clone(),
            "teste".into(),
            true,
            eventos_b,
            parada_b,
        ));
        let mut quadros = [0; 2];
        let mut audios = [0; 2];
        let mut caminhos = [false; 2];
        let resultado = timeout(Duration::from_secs(45), async {
            while quadros.iter().any(|n| *n < 5)
                || audios.iter().any(|n| *n < 5)
                || caminhos.iter().any(|recebido| !recebido)
            {
                let (indice, evento) = tokio::select! {
                    evento = a.recv() => (0, evento),
                    evento = b.recv() => (1, evento),
                };
                let evento =
                    evento.ok_or_else(|| "Canal encerrado antes de receber mídia".to_owned())?;
                match evento.tipo.as_str() {
                    "video_remoto" => {
                        assert_eq!(evento.rgba.len(), 320 * 240 * 4);
                        quadros[indice] += 1;
                    }
                    "audio_teste" => audios[indice] += 1,
                    "caminho_ice" => {
                        assert!(
                            evento.texto.contains(if somente_turn {
                                "retransmissão TURN"
                            } else {
                                "direto"
                            }),
                            "{}",
                            evento.texto
                        );
                        caminhos[indice] = true;
                        eprintln!("Cliente {indice}: {}", evento.texto);
                    }
                    "erro" => return Err(evento.texto),
                    "estado" | "aviso" => eprintln!("Cliente {indice}: {}", evento.texto),
                    _ => {}
                }
            }
            Ok::<(), String>(())
        })
        .await;
        // Mesmo uma falha precisa aguardar a liberação de todos os helpers.
        let _ = parar_a.send(true);
        let _ = parar_b.send(true);
        // Drena quadros enquanto o encerramento publica os eventos finais.
        let limpeza = timeout(Duration::from_secs(10), async {
            loop {
                tokio::select! {
                    evento = a.recv(), if !a.is_closed() => { let _ = evento; },
                    evento = b.recv(), if !b.is_closed() => { let _ = evento; },
                    _ = tokio::time::sleep(Duration::from_millis(20)) => {
                        if tarefa_a.is_finished() && tarefa_b.is_finished() { break; }
                    }
                }
            }
        })
        .await;
        assert!(
            limpeza.is_ok(),
            "O encerramento demorou mais de dez segundos"
        );
        tarefa_a.await.unwrap().unwrap();
        tarefa_b.await.unwrap().unwrap();
        assert!(
            matches!(resultado, Ok(Ok(()))),
            "Rodada {rodada}: {resultado:?}; vídeo={quadros:?}, áudio={audios:?}"
        );
        eprintln!("Rodada {rodada}: vídeo remoto={quadros:?}, blocos PCM remotos={audios:?}");
    }
    servidor.abort();
}
