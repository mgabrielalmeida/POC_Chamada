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

Esta tabela define a arquitetura pretendida. Na base inicial, somente os pontos
de entrada Flutter e Rust existem; as integrações ainda serão implementadas.

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

O próximo trabalho técnico deve comprovar cedo um caminho mínimo de mídia no
Windows e registrar codecs, bibliotecas nativas, versões e comandos necessários.
Ainda não há escolha de codec, resolução exata ou bibliotecas de captura. Evite
adicionar dependências antes de verificar o caminho que elas precisam atender.

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
