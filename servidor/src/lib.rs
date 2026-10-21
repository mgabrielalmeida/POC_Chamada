//! Salas de duas pessoas, mantidas somente em memória.
use anyhow::{Context, Result, bail};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, mpsc},
    time::{Instant, timeout},
};
use tokio_tungstenite::{
    accept_async_with_config,
    tungstenite::{Message, protocol::WebSocketConfig},
};

type Envio = mpsc::Sender<Message>;
type Salas = Arc<Mutex<HashMap<String, Vec<(u64, Envio)>>>>;

fn texto(valor: Value) -> Message {
    Message::Text(valor.to_string().into())
}

/// Aceita conexões até o processo ser encerrado. O chamador escolhe a interface.
pub async fn executar(ouvinte: TcpListener) -> Result<()> {
    let salas: Salas = Arc::default();
    let mut proximo_id = 0;
    loop {
        let (socket, _) = ouvinte.accept().await?;
        proximo_id += 1;
        let salas = salas.clone();
        tokio::spawn(async move {
            if let Err(erro) = atender(socket, salas, proximo_id).await {
                eprintln!("Conexão encerrada: {erro:#}");
            }
        });
    }
}

async fn atender(socket: TcpStream, salas: Salas, id: u64) -> Result<()> {
    let config = WebSocketConfig::default()
        .max_message_size(Some(64 * 1024))
        .max_frame_size(Some(64 * 1024));
    let mut ws = timeout(
        Duration::from_secs(10),
        accept_async_with_config(socket, Some(config)),
    )
    .await??;
    // A primeira mensagem define a sala; nunca confiamos num nome vindo da rede.
    let primeira = timeout(Duration::from_secs(10), ws.next())
        .await?
        .context("Conexão fechada antes de entrar")??;
    let entrada: Value = serde_json::from_str(primeira.to_text()?)?;
    let nome = entrada["sala"].as_str().unwrap_or("");
    if entrada["tipo"] != "entrar"
        || nome.is_empty()
        || nome.len() > 48
        || !nome
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
    {
        ws.send(texto(
            json!({"tipo":"erro","mensagem":"Sala inválida. Use até 48 letras, números, - ou _."}),
        ))
        .await?;
        bail!("Sala inválida");
    }
    let nome = nome.to_owned();
    let (envio, mut recebimento) = mpsc::channel(16);
    {
        let mut mapa = salas.lock().await;
        let pessoas = mapa.entry(nome.clone()).or_default();
        if pessoas.len() == 2 {
            drop(mapa);
            ws.send(texto(
                json!({"tipo":"erro","mensagem":"A sala já tem duas pessoas."}),
            ))
            .await?;
            return Ok(());
        }
        pessoas.push((id, envio.clone()));
        if pessoas.len() == 2 {
            // O primeiro participante oferece; o segundo responde. Evita ofertas simultâneas.
            for (indice, (_, destino)) in pessoas.iter().enumerate() {
                let _ = destino.try_send(texto(json!({"tipo":"pronto","ofertante":indice == 0})));
            }
        } else {
            let _ = envio.try_send(texto(json!({"tipo":"aguardando"})));
        }
    }
    let (mut saida, mut entrada) = ws.split();
    let mut pulso = tokio::time::interval(Duration::from_secs(10));
    let mut ultima_resposta = Instant::now();
    // Qualquer saída passa pelo mesmo bloco de limpeza, inclusive erros de JSON/socket.
    let resultado: Result<()> = async {
        loop {
            tokio::select! {
                Some(mensagem) = recebimento.recv() => {
                    let finalizando = mensagem.is_close();
                    saida.send(mensagem).await?;
                    if finalizando { break; }
                }
                mensagem = entrada.next() => {
                    let Some(mensagem) = mensagem else { break; };
                    let mensagem = mensagem?;
                    ultima_resposta = Instant::now();
                    match mensagem {
                        Message::Text(conteudo) => {
                            let valor: Value = serde_json::from_str(&conteudo)?;
                            if valor["tipo"] != "descricao" || !valor["descricao"].is_object() {
                                bail!("Mensagem de negociação inválida");
                            }
                            let destino = {
                                let mapa = salas.lock().await;
                                mapa.get(&nome).and_then(|p| p.iter().find(|(outro,_)| *outro != id))
                                    .map(|(_,envio)| envio.clone())
                            };
                            if let Some(destino) = destino {
                                destino.try_send(Message::Text(conteudo))?;
                            } else { bail!("A outra pessoa saiu"); }
                        }
                        Message::Close(_) => break,
                        Message::Ping(dados) => saida.send(Message::Pong(dados)).await?,
                        Message::Pong(_) => {},
                        _ => bail!("Somente mensagens de texto são aceitas"),
                    }
                }
                _ = pulso.tick() => {
                    if ultima_resposta.elapsed() > Duration::from_secs(35) { bail!("Conexão sem resposta"); }
                    saida.send(Message::Ping(Vec::new().into())).await?;
                }
            }
        }
        Ok(())
    }.await;
    {
        let mut mapa = salas.lock().await;
        // Remove toda a sala: a pessoa restante também precisa entrar de novo.
        // Conferir o id impede que a limpeza antiga remova uma nova sala de mesmo nome.
        if mapa
            .get(&nome)
            .is_some_and(|p| p.iter().any(|(pessoa, _)| *pessoa == id))
        {
            if let Some(pessoas) = mapa.remove(&nome) {
                for (pessoa, destino) in pessoas {
                    if pessoa != id {
                        let _ = destino.try_send(texto(json!({"tipo":"saiu"})));
                        let _ = destino.try_send(Message::Close(None));
                    }
                }
            }
        }
    }
    let _ = saida.close().await;
    resultado
}
