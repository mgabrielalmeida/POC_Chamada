# Baixa uma versão fixa; não instala serviços nem altera o PATH do computador.
$ErrorActionPreference = 'Stop'
$raizProjeto = Split-Path $PSScriptRoot -Parent
$pastaLocal = Join-Path $raizProjeto '.local'
$arquivoZip = Join-Path $pastaLocal 'ffmpeg.zip'
$pastaFfmpeg = Join-Path $pastaLocal 'ffmpeg-9.0.2-essentials_build'
$hashEsperado = '60F467265B1E312373DBCD92200C2618A74850F98D3D078E94296BB3FA2047BA'
New-Item -ItemType Directory -Force $pastaLocal | Out-Null
if (!(Test-Path -LiteralPath $arquivoZip)) {
    Invoke-WebRequest -Uri 'https://www.gyan.dev/ffmpeg/builds/packages/ffmpeg-9.0.2-essentials_build.zip' -OutFile $arquivoZip
}
if ((Get-FileHash -LiteralPath $arquivoZip -Algorithm SHA256).Hash -ne $hashEsperado) {
    throw 'O SHA256 do FFmpeg não confere. Confira a origem antes de continuar.'
}
if (!(Test-Path -LiteralPath (Join-Path $pastaFfmpeg 'bin/ffmpeg.exe'))) {
    Expand-Archive -LiteralPath $arquivoZip -DestinationPath $pastaLocal
}
Write-Host "FFmpeg conferido em $pastaFfmpeg"
