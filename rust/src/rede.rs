//! Configuração opcional da rede, lida antes de abrir sinalização ou dispositivos.
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use webrtc::peer_connection::{
    RTCConfiguration, RTCConfigurationBuilder, RTCIceServer, RTCIceTransportPolicy,
};

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Rede {
    pub token_sinalizacao: String,
    servidores_ice: Vec<ServidorIce>,
    pub somente_retransmissao: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ServidorIce {
    urls: Vec<String>,
    #[serde(default)]
    usuario: String,
    #[serde(default)]
    senha: String,
}

impl Rede {
    /// Sem variável, mantém a LAN sem exigir serviços externos.
    pub fn carregar() -> Result<Self> {
        let Some(caminho) = std::env::var_os("CHAMADA_CONFIG") else {
            return Ok(Self::default());
        };
        let conteudo = std::fs::read_to_string(caminho)
            .context("Não foi possível ler o arquivo indicado por CHAMADA_CONFIG")?;
        Self::ler(&conteudo)
    }

    fn ler(conteudo: &str) -> Result<Self> {
        // Não inclui a origem do erro serde: ela pode conter um valor secreto.
        serde_json::from_str(conteudo).map_err(|_| {
            anyhow::anyhow!("Configuração de rede inválida. Confira o JSON e os nomes dos campos.")
        })
    }

    pub fn configuracao_ice(&self) -> Result<RTCConfiguration> {
        let mut servidores = Vec::new();
        let mut tem_turn = false;
        for entrada in &self.servidores_ice {
            if entrada.urls.is_empty() {
                bail!("Cada servidor ICE precisa de pelo menos uma URL");
            }
            for url in &entrada.urls {
                // A integração desta POC usa STUN e TURN por UDP. Outros transportes
                // ficam fora até serem ensaiados com a versão fixada de webrtc-rs.
                if !(url.starts_with("stun:") || url.starts_with("turn:"))
                    || url.contains('@')
                    || url.contains('?') && !url.ends_with("?transport=udp")
                {
                    bail!("Use URLs stun: ou turn: por UDP, sem credenciais na URL");
                }
                tem_turn |= url.starts_with("turn:");
            }
            let servidor = RTCIceServer {
                urls: entrada.urls.clone(),
                username: entrada.usuario.clone(),
                credential: entrada.senha.clone(),
            };
            // A biblioteca valida host, porta e credenciais; não expomos o erro
            // original porque a configuração contém segredos locais.
            let urls = servidor.urls().map_err(|_| {
                anyhow::anyhow!("Servidor ICE inválido. Confira URL, usuário e senha TURN")
            })?;
            // webrtc 0.21.1 remonta o endpoint TURN como host:porta sem recolocar
            // os colchetes de um literal IPv6. DNS com AAAA funciona e é o caminho
            // usado na implantação; falhar aqui evita uma coleta silenciosamente vazia.
            if urls
                .iter()
                .any(|url| url.scheme == rtc::ice::url::SchemeType::Turn && url.host.contains(':'))
            {
                bail!(
                    "TURN IPv6 exige hostname com registro AAAA nesta versão; não use IPv6 literal na URL TURN"
                );
            }
            servidores.push(servidor);
        }
        if self.somente_retransmissao && !tem_turn {
            bail!("Retransmissão forçada exige um servidor TURN com credenciais");
        }
        Ok(RTCConfigurationBuilder::default()
            .with_ice_servers(servidores)
            .with_ice_transport_policy(if self.somente_retransmissao {
                RTCIceTransportPolicy::Relay
            } else {
                RTCIceTransportPolicy::All
            })
            .build())
    }

    pub fn validar_sinalizacao(&self, endereco: &str) -> Result<()> {
        let uri: tokio_tungstenite::tungstenite::http::Uri = endereco
            .parse()
            .context("Endereço de sinalização inválido")?;
        let esquema = uri.scheme_str().unwrap_or("");
        if !matches!(esquema, "ws" | "wss") || uri.host().is_none() {
            bail!("Use ws://IP:PORTA na LAN ou wss://DOMINIO na internet");
        }
        let loopback = matches!(uri.host(), Some("localhost" | "127.0.0.1" | "[::1]"));
        if esquema == "ws" && !loopback && !self.token_sinalizacao.is_empty() {
            bail!("O token de sinalização exige wss:// fora do loopback");
        }
        if esquema == "wss" && self.token_sinalizacao.trim().is_empty() {
            bail!("Configure token_sinalizacao para acessar a sinalização externa");
        }
        Ok(())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn lan_permanece_sem_servidores_externos() {
        let rede = Rede::default();
        let ice = rede.configuracao_ice().unwrap();
        assert!(ice.ice_servers().is_empty());
        assert_eq!(ice.ice_transport_policy(), RTCIceTransportPolicy::All);
        rede.validar_sinalizacao("ws://192.168.1.2:8080").unwrap();
    }

    #[test]
    fn turn_com_credenciais_pode_forcar_retransmissao() {
        let rede = Rede::ler(r#"{"servidores_ice":[{"urls":["stun:exemplo.org:3478","turn:exemplo.org:3478?transport=udp"],"usuario":"teste","senha":"segredo"}],"somente_retransmissao":true}"#).unwrap();
        let ice = rede.configuracao_ice().unwrap();
        assert_eq!(ice.ice_transport_policy(), RTCIceTransportPolicy::Relay);
        assert_eq!(ice.ice_servers()[0].username, "teste");
    }

    #[test]
    fn configuracoes_invalidas_falham_sem_expor_segredos() {
        for json in [
            r#"{"somente_retransmissao":true}"#,
            r#"{"servidores_ice":[{"urls":["turn:exemplo.org"]}]}"#,
            r#"{"servidores_ice":[{"urls":[]}]}"#,
            r#"{"servidores_ice":[{"urls":["turn:exemplo.org?transport=tcp"],"usuario":"teste","senha":"segredo"}]}"#,
        ] {
            let erro = Rede::ler(json).unwrap().configuracao_ice().unwrap_err();
            assert!(!erro.to_string().contains("segredo"));
        }
        let erro = Rede::ler(r#"{"campo_incorreto":"segredo"}"#).err().unwrap();
        assert!(!erro.to_string().contains("segredo"));
    }

    #[test]
    fn token_exige_tls_fora_do_loopback() {
        let rede = Rede::ler(r#"{"token_sinalizacao":"teste"}"#).unwrap();
        assert!(rede.validar_sinalizacao("ws://192.168.1.2:8080").is_err());
        rede.validar_sinalizacao("ws://127.0.0.1:8080").unwrap();
        rede.validar_sinalizacao("wss://exemplo.org").unwrap();
        assert!(
            Rede::default()
                .validar_sinalizacao("wss://exemplo.org")
                .is_err()
        );
        assert!(rede.validar_sinalizacao("https://exemplo.org").is_err());
    }

    #[test]
    fn sinalizacao_ipv6_preserva_regras_de_tls_e_token() {
        let rede = Rede::ler(r#"{"token_sinalizacao":"teste"}"#).unwrap();
        rede.validar_sinalizacao("ws://[::1]:8080").unwrap();
        rede.validar_sinalizacao("wss://[2001:db8::1]:443").unwrap();
        rede.validar_sinalizacao("wss://chamada.exemplo.org")
            .unwrap();
        assert!(rede.validar_sinalizacao("ws://[2001:db8::1]:8080").is_err());
        assert!(rede.validar_sinalizacao("wss://[2001:db8::1").is_err());
    }

    #[test]
    fn ice_aceita_dns_aaaa_e_stun_ipv6_mas_explica_limite_do_turn_literal() {
        let rede = Rede::ler(r#"{"servidores_ice":[{"urls":["stun:[2001:db8::1]:3478","turn:chamada.exemplo.org:3478?transport=udp"],"usuario":"teste","senha":"segredo"}]}"#).unwrap();
        rede.configuracao_ice().unwrap();
        let rede = Rede::ler(r#"{"servidores_ice":[{"urls":["turn:[2001:db8::1]:3478?transport=udp"],"usuario":"teste","senha":"segredo"}]}"#).unwrap();
        let erro = rede.configuracao_ice().unwrap_err().to_string();
        assert!(erro.contains("AAAA"));
        assert!(!erro.contains("segredo"));
    }
}
