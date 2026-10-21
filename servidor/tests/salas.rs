//! Testes reais de WebSocket, sem câmera, microfone ou portas fixas.
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{
    net::{TcpListener, TcpStream},
    time::{Duration, timeout},
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

type Cliente = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn receber(cliente: &mut Cliente) -> Value {
    timeout(Duration::from_secs(5), async {
        loop {
            match cliente.next().await.unwrap().unwrap() {
                Message::Text(texto) => return serde_json::from_str(&texto).unwrap(),
                Message::Ping(dados) => cliente.send(Message::Pong(dados)).await.unwrap(),
                outra => panic!("Mensagem inesperada: {outra:?}"),
            }
        }
    })
    .await
    .expect("Servidor não respondeu")
}

async fn entrar(endereco: &str, sala: &str) -> Cliente {
    let (mut cliente, _) = connect_async(endereco).await.unwrap();
    cliente
        .send(Message::Text(
            json!({"tipo":"entrar", "sala":sala}).to_string().into(),
        ))
        .await
        .unwrap();
    cliente
}

#[tokio::test]
async fn duas_pessoas_negociam_terceira_e_rejeitada_e_sala_pode_ser_reutilizada() {
    let ouvinte = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endereco = format!("ws://{}", ouvinte.local_addr().unwrap());
    let servidor = tokio::spawn(sinalizacao::executar(ouvinte));
    let mut primeira = entrar(&endereco, "teste").await;
    assert_eq!(receber(&mut primeira).await["tipo"], "aguardando");
    let mut segunda = entrar(&endereco, "teste").await;
    assert_eq!(receber(&mut primeira).await["ofertante"], true);
    assert_eq!(receber(&mut segunda).await["ofertante"], false);
    let mut terceira = entrar(&endereco, "teste").await;
    assert_eq!(receber(&mut terceira).await["tipo"], "erro");
    let oferta = json!({"tipo":"descricao","descricao":{"type":"offer","sdp":"exemplo"}});
    primeira
        .send(Message::Text(oferta.to_string().into()))
        .await
        .unwrap();
    assert_eq!(receber(&mut segunda).await, oferta);
    primeira.close(None).await.unwrap();
    assert_eq!(receber(&mut segunda).await["tipo"], "saiu");
    let mut nova = entrar(&endereco, "teste").await;
    assert_eq!(receber(&mut nova).await["tipo"], "aguardando");
    let _ = nova.close(None).await;
    servidor.abort();
}

#[tokio::test]
async fn sala_invalida_e_mensagem_invalida_nao_deixam_participante_preso() {
    let ouvinte = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endereco = format!("ws://{}", ouvinte.local_addr().unwrap());
    let servidor = tokio::spawn(sinalizacao::executar(ouvinte));
    let mut invalida = entrar(&endereco, "nome com espaço").await;
    assert_eq!(receber(&mut invalida).await["tipo"], "erro");
    let mut primeira = entrar(&endereco, "limpeza").await;
    receber(&mut primeira).await;
    let mut segunda = entrar(&endereco, "limpeza").await;
    receber(&mut primeira).await;
    receber(&mut segunda).await;
    primeira
        .send(Message::Text("JSON inválido".into()))
        .await
        .unwrap();
    assert_eq!(receber(&mut segunda).await["tipo"], "saiu");
    let mut nova = entrar(&endereco, "limpeza").await;
    assert_eq!(receber(&mut nova).await["tipo"], "aguardando");
    let _ = nova.close(None).await;
    servidor.abort();
}

#[tokio::test]
async fn token_invalido_nao_ocupa_sala_e_token_valido_permite_negociar() {
    let ouvinte = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endereco = format!("ws://{}", ouvinte.local_addr().unwrap());
    let servidor = tokio::spawn(sinalizacao::executar_com_token(
        ouvinte,
        Some("token-do-teste".into()),
    ));
    for token in [None, Some("incorreto")] {
        let (mut cliente, _) = connect_async(&endereco).await.unwrap();
        cliente
            .send(Message::Text(
                json!({"tipo":"entrar","sala":"restrita","token":token})
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        let resposta = receber(&mut cliente).await;
        assert_eq!(resposta["tipo"], "erro");
        assert_eq!(resposta["mensagem"], "Acesso não autorizado.");
        let _ = cliente.close(None).await;
    }
    let mut clientes = Vec::new();
    for _ in 0..2 {
        let (mut cliente, _) = connect_async(&endereco).await.unwrap();
        cliente
            .send(Message::Text(
                json!({"tipo":"entrar","sala":"restrita","token":"token-do-teste"})
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        clientes.push(cliente);
        if clientes.len() == 1 {
            assert_eq!(receber(&mut clientes[0]).await["tipo"], "aguardando");
        }
    }
    assert_eq!(receber(&mut clientes[0]).await["ofertante"], true);
    assert_eq!(receber(&mut clientes[1]).await["ofertante"], false);
    let oferta = json!({"tipo":"descricao","descricao":{"type":"offer","sdp":"teste autorizado"}});
    clientes[0]
        .send(Message::Text(oferta.to_string().into()))
        .await
        .unwrap();
    assert_eq!(receber(&mut clientes[1]).await, oferta);
    clientes[0].close(None).await.unwrap();
    assert_eq!(receber(&mut clientes[1]).await["tipo"], "saiu");
    servidor.abort();
}

#[tokio::test]
async fn token_vazio_nao_desativa_protecao_silenciosamente() {
    let ouvinte = TcpListener::bind("127.0.0.1:0").await.unwrap();
    assert!(
        sinalizacao::executar_com_token(ouvinte, Some(" ".into()))
            .await
            .is_err()
    );
}
