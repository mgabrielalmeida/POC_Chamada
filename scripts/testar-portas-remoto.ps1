# Testa, a partir de uma rede EXTERNA, se as portas da implantação chegam ao
# host candidato. Usar com scripts/escutar-teste-portas.py rodando no host.
#
# Uso:
#   .\scripts\testar-portas-remoto.ps1 -Enderecos '2001:db8::1','203.0.113.10'

param(
    [Parameter(Mandatory)] [string[]] $Enderecos,
    [int[]] $PortasTcp = @(80, 443),
    [int[]] $PortasUdp = @(3478, 49160, 49200),
    [int] $TempoLimiteMs = 3000
)

function Testar-Tcp($ip, $porta) {
    $cliente = New-Object System.Net.Sockets.TcpClient($ip.AddressFamily)
    try {
        if (-not $cliente.ConnectAsync($ip, $porta).Wait($TempoLimiteMs)) { return 'sem resposta' }
        $fluxo = $cliente.GetStream()
        $fluxo.ReadTimeout = $TempoLimiteMs
        $buffer = New-Object byte[] 64
        $lidos = $fluxo.Read($buffer, 0, $buffer.Length)
        return [Text.Encoding]::ASCII.GetString($buffer, 0, $lidos).Trim()
    } catch { return "falhou: $($_.Exception.InnerException.Message)" }
    finally { $cliente.Close() }
}

function Testar-Udp($ip, $porta) {
    $cliente = New-Object System.Net.Sockets.UdpClient($ip.AddressFamily)
    try {
        $cliente.Client.ReceiveTimeout = $TempoLimiteMs
        $dados = [Text.Encoding]::ASCII.GetBytes('teste')
        [void]$cliente.Send($dados, $dados.Length, (New-Object System.Net.IPEndPoint($ip, $porta)))
        $origem = New-Object System.Net.IPEndPoint($ip, 0)
        return [Text.Encoding]::ASCII.GetString($cliente.Receive([ref]$origem))
    } catch { return 'sem resposta' }
    finally { $cliente.Close() }
}

foreach ($texto in $Enderecos) {
    $ip = [System.Net.IPAddress]::Parse($texto)
    foreach ($porta in $PortasTcp) { '{0,-40} TCP {1,-5} {2}' -f $texto, $porta, (Testar-Tcp $ip $porta) }
    foreach ($porta in $PortasUdp) { '{0,-40} UDP {1,-5} {2}' -f $texto, $porta, (Testar-Udp $ip $porta) }
}
