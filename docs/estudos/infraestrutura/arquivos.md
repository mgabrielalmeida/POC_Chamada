# Infraestrutura: leitura arquivo a arquivo

[Funcionamento geral](README.md) · [Configuração operacional](../../desenvolvimento/etapa-2-configuracao.md)

## config/cliente.exemplo.json

O [modelo do cliente](../../../config/cliente.exemplo.json) contém o contrato
consumido por `Rede`: `token_sinalizacao`, lista `servidores_ice` e booleano
`somente_retransmissao`. Cada servidor tem `urls`, `usuario` e `senha`.
A entrada de exemplo contém uma URL STUN e uma TURN/UDP para o mesmo hostname.

Os textos de substituição não são credenciais válidas. A cópia preenchida fica
fora do Git. O campo da interface continua recebendo a URL WSS separadamente;
o JSON não contém sala nem URL de sinalização.

Alterar `somente_retransmissao` muda a política ICE na próxima entrada. Não
altera em tempo real a PeerConnection já criada. A validação dos campos e as
mensagens de erro estão em [rede.rs](../webrtc/arquivos.md).

## config/Caddyfile.exemplo

O [Caddyfile](../../../config/Caddyfile.exemplo) declara um domínio de exemplo
e `reverse_proxy 127.0.0.1:8080`. Sua finalidade é expor a sinalização com
HTTPS/WSS e encaminhar o WebSocket ao backend da mesma máquina.

É pequeno porque não contém a lógica de salas e não configura TURN. O domínio,
DNS e condições de emissão do certificado precisam existir na implantação.
Ter esse arquivo no repositório não inicia Caddy nem emite certificado.

## config/turnserver-ipv6.exemplo.conf

O [modelo coturn](../../../config/turnserver-ipv6.exemplo.conf) é voltado ao
coturn 4.18.0 nativo Windows e precisa ser preenchido com o IPv6 global real.
Leia por grupos de responsabilidade:

| Grupo | Campos e motivo |
| --- | --- |
| Endereços | `listening-ip` recebe pedidos; `relay-ip` fornece a origem das alocações |
| Família | `allocation-default-address-family=ipv6` atende o pedido sem atributo de família do cliente fixado |
| Autenticação | `lt-cred-mech`, `realm` e `user` configuram credenciais TURN; são diferentes do token da sala |
| Portas | 3478 para o serviço e 49160–49200 para as alocações; o firewall precisa permitir o efeito pretendido |
| Quotas | `user-quota=8` e `total-quota=16` limitam alocações; não representam diretamente quantidade de pessoas na chamada |
| Transportes | `no-tcp`, `no-tls` e `no-tcp-relay` limitam o desenho aos caminhos UDP usados |
| Destinos | Filtros negam faixas privadas/locais e multicast; não liberar loopback no serviço público |
| Diagnóstico | `log-file=stdout` entrega logs à saída do processo |

Não há `external-ip`, pois esse exemplo não descreve mapeamento NAT IPv4.
Também não se deve acrescentar `denied-peer-ip=::`: o histórico mostrou que
essa regra podia bloquear todos os peers IPv6. `::1` e intervalos específicos
são diferentes desse limite curinga.

A configuração de coturn isolado para testes em loopback está na
[auditoria](../../desenvolvimento/auditoria-ipv6.md). Sua exceção
`allow-loopback-peers` pertence apenas ao processo local de teste, não a este
modelo público. Misturar os dois arquivos mudaria o escopo de acesso do serviço.

## app/windows/rust.cmake

[rust.cmake](../../../app/windows/rust.cmake) resolve a raiz do workspace e
define o perfil Cargo conforme a configuração CMake. Debug usa perfil `dev`
e saída `target/debug`; os demais usam `release` e `target/release`.

Cria o alvo `chamada_rust`, executado no build, com `cargo build -p chamada`.
O alvo do executável Flutter depende dele. Em seguida, as regras de instalação
copiam `chamada.dll` para junto do executável.

O mesmo arquivo instala `ffmpeg.exe`, `ffplay.exe`, `LICENSE` e `README.txt`
da distribuição preparada em `.local`. As licenças vão para
`licencas/ffmpeg`. Esse build não baixa FFmpeg: a preparação deve ter ocorrido
antes. O [script de preparação](../midia/arquivos.md) explica download/hash.

O comando Cargo do CMake não passa `--locked`; a rotina de verificação separada
usa essa opção para conferir a resolução versionada. Não confunda uma opção
do guia de testes com um argumento já presente no build CMake.

## Fontes e arquivos de apoio

O [Cargo.lock](../../../Cargo.lock) e o [pubspec.lock](../../../app/pubspec.lock)
registram resoluções; não substituem executáveis de toolchain ou dependências
nativas. A função dos manifestos e dos arquivos Windows restantes está nos
módulos de [arquitetura](../arquitetura/README.md) e [Flutter](../flutter/arquivos.md).

As fontes primárias preservadas dos guias originais ajudam a aprofundar cada
fronteira: [configuração do gerador FRB](https://cjycode.com/flutter_rust_bridge/guides/custom/codegen/inputs),
[DirectShow no FFmpeg](https://ffmpeg.org/ffmpeg-devices.html#dshow),
[distribuições Windows indicadas pelo FFmpeg](https://ffmpeg.org/download.html#build-windows)
e [workflow MSVC coturn 4.18.0](https://github.com/coturn/coturn/blob/4.18.0/.github/workflows/msvc.yml).
Use exemplos de webrtc-rs compatíveis com a versão fixada; a branch principal
upstream pode evoluir sem que a POC tenha sido atualizada.
