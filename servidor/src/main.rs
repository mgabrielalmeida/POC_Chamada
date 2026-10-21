//! Sinalização LAN ou backend local de um proxy WSS, sem TLS duplicado em Rust.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = match std::env::var("SINALIZACAO_TOKEN") {
        Ok(valor) if valor.trim().is_empty() => {
            anyhow::bail!("SINALIZACAO_TOKEN não pode ser vazio")
        }
        Ok(valor) => Some(valor),
        Err(std::env::VarError::NotPresent) => None,
        Err(_) => anyhow::bail!("SINALIZACAO_TOKEN precisa ser texto Unicode válido"),
    };
    let endereco = std::env::args().nth(1).unwrap_or_else(|| {
        if token.is_some() {
            "127.0.0.1:8080"
        } else {
            "0.0.0.0:8080"
        }
        .into()
    });
    let ouvinte = tokio::net::TcpListener::bind(&endereco).await?;
    println!(
        "Sinalização em ws://{}; acesso {}. Ctrl+C encerra.",
        ouvinte.local_addr()?,
        if token.is_some() {
            "com token (use proxy WSS para internet)"
        } else {
            "LAN sem token"
        }
    );
    tokio::select! {
        resultado = sinalizacao::executar_com_token(ouvinte, token) => resultado?,
        _ = tokio::signal::ctrl_c() => {},
    }
    Ok(())
}
