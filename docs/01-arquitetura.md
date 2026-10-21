# Arquitetura e decisões

## Responsabilidades

| Parte | Responsabilidade |
| --- | --- |
| Flutter | Mostrar controles, estados e quadros de vídeo; encaminhar ações do usuário. |
| Flutter-Rust Bridge | Transportar comandos e resultados entre Dart e Rust. |
| Rust do cliente | Captura, codecs, áudio, sinalização, WebRTC e ciclo de vida da chamada. |
| webrtc-rs | Implementar a conexão WebRTC dentro da camada Rust. Uso obrigatório. |
| Servidor Rust | Reunir duas pessoas em uma sala e encaminhar a negociação. |
| TURN próprio | Retransmitir mídia quando necessário na etapa entre redes diferentes. |

Na Etapa 1, Flutter-Rust Bridge conecta a interface à biblioteca nativa. O cliente
usa `webrtc` 0.21.1 e seus tipos de protocolo `rtc` 0.21.1. FFmpeg 9.0.2 fornece
captura e codecs como dependência auxiliar, sempre iniciada e encerrada por Rust.

## Caminho de uma chamada

1. A pessoa informa o servidor e cria ou entra em uma sala na interface.
2. Flutter encaminha a ação para Rust usando a ponte.
3. Rust troca mensagens de negociação com o servidor de sinalização.
4. Os clientes Rust estabelecem a conexão WebRTC e transportam áudio e vídeo.
5. Rust reproduz áudio e entrega estados e quadros decodificados ao Flutter.
6. Ao encerrar, Rust fecha conexões e libera câmera e microfone.

Sinalização e transporte de mídia têm papéis distintos. A escolha inicial é
WebSocket para sinalização, com salas de no máximo duas pessoas em memória.
O formato das mensagens será definido ao implementar esse fluxo, sem criar agora
um pacote compartilhado ou uma camada adicional de abstração.

## Mídia é um trabalho explícito

A captura, a codificação, a decodificação e a reprodução precisam ser integradas
na camada Rust. A POC não deve considerar a escolha de WebRTC como implementação
concluída dessas funções. Bibliotecas auxiliares são permitidas se preservarem
essa divisão de responsabilidades.

O vídeo usa VP8, 320×240 e 10 quadros por segundo; áudio usa Opus, 48 kHz e dois
canais. O caminho detalhado está no guia da Etapa 1. Usamos a primeira câmera e
o primeiro microfone enumerados por DirectShow, sem seleção na tela. Isso pode
diferir do microfone preferido nas configurações do Windows.

## Escopo fechado

- Apenas Windows com Windows, duas pessoas e dispositivos padrão.
- Qualidade modesta, priorizando funcionamento e simplicidade.
- Interface mínima: criar/entrar em sala, vídeos, estado e encerrar.
- Sem contas, banco de dados, histórico, seleção de dispositivos, reconexão
  automática, chamadas em grupo ou tratamento avançado de áudio nesta versão.
- Código próprio comentado em português, explicando decisões e liberação de
  recursos. Nomes exigidos por ferramentas e código gerado podem permanecer no original.
- Sem camadas genéricas, infraestrutura de produção ou pacotes sem uso concreto.

## Infraestrutura e pontos ainda abertos

Na rede local, um computador pode hospedar a sinalização. Na segunda etapa será
usada uma máquina disponível, sem contratação de serviços externos. Não há limite
informado de tráfego ou uso. Sistema operacional, acesso administrativo, IP
público, NAT e regras de firewall dessa máquina ainda precisam ser confirmados.

TURN será auto-hospedado, se necessário, usando a infraestrutura existente. A
implementação escolhida e seu modo de execução dependem da inspeção da máquina;
não se assume que ela execute um serviço TURN específico nativamente no Windows.
O servidor de sinalização da POC será escrito em Rust; TURN é um serviço separado.
Endereços, portas, credenciais e proteção do acesso serão definidos antes de
expor os serviços à internet.
