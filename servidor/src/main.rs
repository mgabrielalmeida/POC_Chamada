//! Servidor LAN. O endereço pode ser informado como primeiro argumento.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endereco = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:8080".into());
    let ouvinte = tokio::net::TcpListener::bind(&endereco).await?;
    println!(
        "Sinalização LAN em ws://{}. Ctrl+C encerra.",
        ouvinte.local_addr()?
    );
    tokio::select! {
        resultado = sinalizacao::executar(ouvinte) => resultado?,
        _ = tokio::signal::ctrl_c() => {},
    }
    Ok(())
}
