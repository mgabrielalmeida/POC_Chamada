//! Captura e codecs em processos FFmpeg, sempre criados e encerrados por Rust.
//! O RTP nestes sockets fica restrito ao loopback; a rede entre PCs usa webrtc-rs.
use anyhow::{Context, Result, bail};
use std::{
    net::{SocketAddr, UdpSocket as SocketPadrao},
    path::PathBuf,
    process::Stdio,
};
use tempfile::TempDir;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::UdpSocket,
    process::{Child, Command},
    sync::mpsc,
    task::JoinSet,
};

pub const LARGURA: u32 = 320;
pub const ALTURA: u32 = 240;

/// O Windows mata os helpers se o aplicativo for fechado abruptamente. Sem isso,
/// `kill_on_drop` só protegeria caminhos em que os destrutores Rust são executados.
#[cfg(windows)]
struct TrabalhoWindows(isize);

#[cfg(windows)]
impl TrabalhoWindows {
    fn novo() -> Result<Self> {
        use windows_sys::Win32::System::JobObjects::*;
        // São estruturas C sem referências; zerar inicializa os campos opcionais.
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            let trabalho = Self(handle as isize);
            let mut limites: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limites.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &limites as *const _ as *const _,
                std::mem::size_of_val(&limites) as u32,
            ) == 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            Ok(trabalho)
        }
    }

    fn associar(&self, filho: &Child) -> Result<()> {
        let handle = filho
            .raw_handle()
            .context("Processo de mídia sem handle Windows")?;
        // O handle pertence ao Child e só é usado enquanto ele está vivo.
        let sucesso = unsafe {
            windows_sys::Win32::System::JobObjects::AssignProcessToJobObject(
                self.0 as _,
                handle as _,
            )
        };
        if sucesso == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(())
    }
}

#[cfg(windows)]
impl Drop for TrabalhoWindows {
    fn drop(&mut self) {
        // Fechar o último handle encerra qualquer helper ainda associado.
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0 as _);
        }
    }
}

/// Eventos internos também permitem testar mídia sem inicializar o Flutter.
#[derive(Debug)]
pub enum EventoMidia {
    Quadro { local: bool, rgba: Vec<u8> },
    AudioDecodificado,
    Erro(String),
}

/// Possui todos os processos e tarefas de uma sessão; o encerramento é explícito.
pub struct Midia {
    pub video: std::sync::Arc<UdpSocket>,
    pub audio: std::sync::Arc<UdpSocket>,
    pub destino_video: SocketAddr,
    pub destino_audio: SocketAddr,
    _rtcp_captura: Vec<SocketPadrao>,
    filhos: Vec<Child>,
    tarefas: JoinSet<()>,
    _temporarios: TempDir,
    #[cfg(windows)]
    trabalho: TrabalhoWindows,
}

/// Localiza binários junto ao aplicativo ou no diretório de desenvolvimento.
pub fn binario(nome: &str) -> Result<PathBuf> {
    let junto = std::env::current_exe()?
        .parent()
        .context("Executável sem pasta")?
        .join(nome);
    if junto.is_file() {
        return Ok(junto);
    }
    let desenvolvimento = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../.local/ffmpeg-9.0.2-essentials_build/bin")
        .join(nome);
    if desenvolvimento.is_file() {
        return Ok(desenvolvimento);
    }
    bail!("{nome} não encontrado. Execute scripts/preparar-midia.ps1 e recompile o aplicativo.");
}

fn comando(nome: &str) -> Result<Command> {
    let mut comando = Command::new(binario(nome)?);
    comando
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    // Cada helper é invisível. A única janela da POC pertence ao Flutter.
    #[cfg(windows)]
    comando.creation_flags(0x08000000);
    Ok(comando)
}

/// DirectShow não define uma câmera padrão global. A POC usa a primeira câmera
/// e o primeiro microfone enumerados, sem tela de seleção nesta etapa.
async fn dispositivos() -> Result<(String, String)> {
    let saida = comando("ffmpeg.exe")?
        .args([
            "-hide_banner",
            "-list_devices",
            "true",
            "-f",
            "dshow",
            "-i",
            "dummy",
        ])
        .output()
        .await?;
    // A listagem usa stderr e pode terminar com código de erro mesmo com dispositivos.
    let texto = String::from_utf8_lossy(&saida.stderr);
    let localizar = |tipo: &str| -> Option<String> {
        texto
            .lines()
            .find(|linha| linha.ends_with(tipo))
            .and_then(|linha| {
                let inicio = linha.find('"')? + 1;
                let fim = linha.rfind('"')?;
                Some(linha[inicio..fim].to_owned())
            })
    };
    Ok((
        localizar("(video)")
            .context("Nenhuma câmera DirectShow disponível. Confira as permissões do Windows.")?,
        localizar("(audio)")
            .context("Nenhum microfone DirectShow disponível. Confira as permissões do Windows.")?,
    ))
}

/// Reserva um par RTP/RTCP até o processo receptor ser iniciado.
fn portas_recepcao() -> Result<(SocketPadrao, SocketPadrao)> {
    for _ in 0..100 {
        // Fora da faixa efêmera padrão do Windows, para não disputar uma porta
        // anunciada ao decoder com os sockets de saída dos próprios helpers.
        let porta = 20000 + (rand::random::<u16>() % 10000) * 2;
        let Ok(rtp) = SocketPadrao::bind((std::net::Ipv4Addr::LOCALHOST, porta)) else {
            continue;
        };
        let porta = rtp.local_addr()?.port();
        if porta == u16::MAX {
            continue;
        }
        if let Ok(rtcp) = SocketPadrao::bind((std::net::Ipv4Addr::LOCALHOST, porta + 1)) {
            return Ok((rtp, rtcp));
        }
    }
    bail!("Não foi possível reservar portas locais de mídia")
}

impl Midia {
    pub async fn iniciar(eventos: mpsc::Sender<EventoMidia>, sintetica: bool) -> Result<Self> {
        // Reservar o vizinho RTCP evita colisão com o RTP de outra trilha.
        let (video, rtcp_video) = portas_recepcao()?;
        let (audio, rtcp_audio) = portas_recepcao()?;
        video.set_nonblocking(true)?;
        audio.set_nonblocking(true)?;
        let video = std::sync::Arc::new(UdpSocket::from_std(video)?);
        let audio = std::sync::Arc::new(UdpSocket::from_std(audio)?);
        let (reserva_v, reserva_v_rtcp) = portas_recepcao()?;
        let (reserva_a, reserva_a_rtcp) = portas_recepcao()?;
        let destino_video = reserva_v.local_addr()?;
        let destino_audio = reserva_a.local_addr()?;
        let temporarios = tempfile::tempdir()?;
        let sdp_video = temporarios.path().join("video.sdp");
        let sdp_audio = temporarios.path().join("audio.sdp");
        for (arquivo, endereco, tipo, payload, codec) in [
            (&sdp_video, destino_video, "video", 96, "VP8/90000"),
            (&sdp_audio, destino_audio, "audio", 111, "opus/48000/2"),
        ] {
            std::fs::write(
                arquivo,
                format!(
                    "v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=POC\r\nc=IN IP4 127.0.0.1\r\nt=0 0\r\nm={tipo} {} RTP/AVP {payload}\r\na=rtpmap:{payload} {codec}\r\na=recvonly\r\n",
                    endereco.port()
                ),
            )?;
        }
        let mut midia = Self {
            video,
            audio,
            destino_video,
            destino_audio,
            _rtcp_captura: vec![rtcp_video, rtcp_audio],
            filhos: Vec::new(),
            tarefas: JoinSet::new(),
            _temporarios: temporarios,
            #[cfg(windows)]
            trabalho: TrabalhoWindows::novo()?,
        };

        // Receptores são preparados antes da conexão para não perder o primeiro keyframe.
        let mut decodificador = comando("ffmpeg.exe")?;
        decodificador
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-nostdin",
                "-protocol_whitelist",
                "file,udp,rtp",
                "-localaddr",
                "127.0.0.1",
                "-flags",
                "low_delay",
                "-analyzeduration",
                "100000",
                "-probesize",
                "32",
                "-f",
                "sdp",
                "-i",
            ])
            .arg(&sdp_video)
            .args([
                "-an",
                "-vf",
                "scale=320:240",
                "-pix_fmt",
                "rgba",
                "-fps_mode",
                "passthrough",
                "-f",
                "rawvideo",
                "pipe:1",
            ])
            .stdout(Stdio::piped());
        drop((reserva_v, reserva_v_rtcp));
        midia
            .adicionar(decodificador, Some(false), eventos.clone())
            .await?;

        let mut reprodutor = comando(if sintetica {
            "ffmpeg.exe"
        } else {
            "ffplay.exe"
        })?;
        reprodutor.args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-protocol_whitelist",
            "file,udp,rtp",
            "-localaddr",
            "127.0.0.1",
            "-analyzeduration",
            "100000",
            "-probesize",
            "32",
        ]);
        if sintetica {
            // O teste lê PCM decodificado sem tocar som nos alto-falantes.
            reprodutor
                .args(["-nostdin", "-f", "sdp", "-i"])
                .arg(&sdp_audio)
                .args(["-f", "s16le", "-ac", "2", "-ar", "48000", "pipe:1"])
                .stdout(Stdio::piped());
        } else {
            reprodutor
                .args([
                    "-nodisp",
                    "-autoexit",
                    "-fflags",
                    "nobuffer",
                    "-flags",
                    "low_delay",
                    "-i",
                ])
                .arg(&sdp_audio);
        }
        drop((reserva_a, reserva_a_rtcp));
        let indice_audio = midia.filhos.len();
        midia.adicionar(reprodutor, None, eventos.clone()).await?;
        if sintetica {
            let mut pcm = midia.filhos[indice_audio]
                .stdout
                .take()
                .context("Saída PCM ausente")?;
            let eventos = eventos.clone();
            midia.tarefas.spawn(async move {
                let mut bloco = vec![0; 3840];
                while pcm.read_exact(&mut bloco).await.is_ok() {
                    let _ = eventos.try_send(EventoMidia::AudioDecodificado);
                }
            });
        }

        let dispositivos = if sintetica {
            None
        } else {
            Some(dispositivos().await?)
        };
        let mut camera = comando("ffmpeg.exe")?;
        camera.args(["-hide_banner", "-loglevel", "error", "-nostdin"]);
        if let Some((camera_nome, _)) = &dispositivos {
            camera
                .args(["-f", "dshow", "-rtbufsize", "32M", "-i"])
                .arg(format!("video={camera_nome}"));
        } else {
            camera.args(["-re", "-f", "lavfi", "-i", "testsrc2=size=320x240:rate=10"]);
        }
        camera
            .args([
                "-map",
                "0:v",
                "-an",
                "-vf",
                "scale=320:240,fps=10",
                "-pix_fmt",
                "rgba",
                "-f",
                "rawvideo",
                "pipe:1",
                "-map",
                "0:v",
                "-an",
                "-vf",
                "scale=320:240,fps=10",
                "-c:v",
                "libvpx",
                "-deadline",
                "realtime",
                "-cpu-used",
                "8",
                "-lag-in-frames",
                "0",
                "-g",
                "10",
                "-b:v",
                "300k",
                "-pix_fmt",
                "yuv420p",
                "-payload_type",
                "96",
                "-f",
                "rtp",
            ])
            .arg(format!(
                "rtp://{}?pkt_size=1200&localaddr=127.0.0.1&localrtpport=0&localrtcpport=0",
                midia.video.local_addr()?
            ))
            .stdout(Stdio::piped());
        midia.adicionar(camera, Some(true), eventos.clone()).await?;

        let mut microfone = comando("ffmpeg.exe")?;
        microfone.args(["-hide_banner", "-loglevel", "error", "-nostdin"]);
        if let Some((_, nome)) = &dispositivos {
            microfone
                .args(["-f", "dshow", "-audio_buffer_size", "50", "-i"])
                .arg(format!("audio={nome}"));
        } else {
            microfone.args([
                "-re",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000",
            ]);
        }
        microfone
            .args([
                "-vn",
                "-ac",
                "2",
                "-ar",
                "48000",
                "-c:a",
                "libopus",
                "-b:a",
                "32k",
                "-application",
                "voip",
                "-frame_duration",
                "20",
                "-payload_type",
                "111",
                "-f",
                "rtp",
            ])
            .arg(format!(
                "rtp://{}?pkt_size=1200&localaddr=127.0.0.1&localrtpport=0&localrtcpport=0",
                midia.audio.local_addr()?
            ));
        midia.adicionar(microfone, None, eventos).await?;
        Ok(midia)
    }

    async fn adicionar(
        &mut self,
        mut comando: Command,
        video_local: Option<bool>,
        eventos: mpsc::Sender<EventoMidia>,
    ) -> Result<()> {
        let mut filho = comando
            .spawn()
            .context("Não foi possível iniciar o helper de mídia")?;
        #[cfg(windows)]
        self.trabalho.associar(&filho)?;
        if let Some(local) = video_local {
            let mut saida = filho.stdout.take().context("Saída de vídeo ausente")?;
            let eventos = eventos.clone();
            self.tarefas.spawn(async move {
                let mut quadro = vec![0; (LARGURA * ALTURA * 4) as usize];
                while saida.read_exact(&mut quadro).await.is_ok() {
                    // Descarta quadros se a interface estiver atrasada; nunca acumula vídeo.
                    let _ = eventos.try_send(EventoMidia::Quadro {
                        local,
                        rgba: quadro.clone(),
                    });
                }
            });
        }
        let erros = filho
            .stderr
            .take()
            .context("Saída de diagnóstico ausente")?;
        self.tarefas.spawn(async move {
            let mut linhas = BufReader::new(erros).lines();
            while let Ok(Some(linha)) = linhas.next_line().await {
                // O diagnóstico do helper pode estar em inglês; preservamos a mensagem original.
                let _ = eventos.try_send(EventoMidia::Erro(linha));
            }
        });
        self.filhos.push(filho);
        Ok(())
    }

    /// Detecta falhas de captura/reprodução mesmo quando o processo não gerou stderr.
    pub fn verificar_processos(&mut self) -> Result<()> {
        for filho in &mut self.filhos {
            if let Some(estado) = filho.try_wait()? {
                bail!("Um processo de mídia encerrou: {estado}");
            }
        }
        Ok(())
    }

    pub async fn encerrar(&mut self) {
        for filho in &mut self.filhos {
            let _ = filho.kill().await;
            let _ = filho.wait().await;
        }
        self.tarefas.abort_all();
        while self.tarefas.join_next().await.is_some() {}
    }
}
