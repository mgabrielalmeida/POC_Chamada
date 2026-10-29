#!/usr/bin/env bash
# Avaliação somente leitura de uma máquina Linux candidata a hospedar
# sinalização (Caddy + Rust) e coturn. Não instala, não altera configuração
# e não abre portas. Comandos privilegiados usam `sudo -n` e são pulados
# quando não há sudo sem senha.
#
# Uso:  bash avaliar-host-linux.sh > avaliacao-$(hostname).txt 2>&1
#
# A saída contém endereços IP da máquina. Revise antes de compartilhar e
# não a versione no Git.

set -u

secao() { printf '\n===== %s =====\n' "$1"; }
tem() { command -v "$1" >/dev/null 2>&1; }
rodar() { printf '$ %s\n' "$*"; "$@" 2>&1 || printf '(saída %s)\n' "$?"; }
privilegiado() {
  if sudo -n true 2>/dev/null; then rodar sudo -n "$@"
  else printf '$ sudo %s\n(pulado: sem sudo sem senha)\n' "$*"; fi
}

secao "Identificação"
rodar date -Is
rodar hostname -f
[ -r /etc/os-release ] && rodar cat /etc/os-release
rodar uname -srm
rodar uptime
tem who && rodar who

secao "Recursos"
rodar nproc
rodar free -h
rodar df -h / /var /opt /srv 2>/dev/null

secao "Privilégios"
rodar id
if sudo -n true 2>/dev/null; then echo "sudo sem senha: SIM"
else echo "sudo sem senha: NÃO (pode haver sudo com senha; testar manualmente com 'sudo -v')"; fi

secao "Init e relógio"
rodar ps -p 1 -o comm=
tem systemctl && rodar systemctl is-system-running
tem timedatectl && rodar timedatectl
tem systemctl && rodar systemctl status sleep.target suspend.target --no-pager

secao "Interfaces e rotas"
rodar ip -br addr
rodar ip -6 addr show scope global
rodar ip route show default
rodar ip -6 route show default

secao "Endereço público visto de fora"
if tem curl; then
  printf 'IPv4 público: '; curl -4 -s --max-time 8 https://api64.ipify.org || echo "(falhou)"; echo
  printf 'IPv6 público: '; curl -6 -s --max-time 8 https://api64.ipify.org || echo "(falhou)"; echo
else
  echo "curl ausente"
fi
echo "Compare com 'ip -br addr': IPv4 público igual ao da placa => sem NAT;"
echo "diferente ou placa em 10.x/172.16-31.x/192.168.x/100.64.x => atrás de NAT."

secao "Proxy e DNS"
env | grep -i '_proxy=' || echo "nenhuma variável de proxy"
[ -r /etc/resolv.conf ] && rodar grep -v '^#' /etc/resolv.conf
tem getent && rodar getent ahosts github.com

secao "Portas em uso (conflitos com 80, 443, 3478, 8080, 49160-49200)"
if tem ss; then
  rodar ss -tulnH
  privilegiado ss -tulpnH
fi

secao "Firewall local"
tem ufw && privilegiado ufw status verbose
tem firewall-cmd && rodar firewall-cmd --state && privilegiado firewall-cmd --list-all
tem nft && privilegiado nft list ruleset
tem iptables && privilegiado iptables -S
tem ip6tables && privilegiado ip6tables -S

secao "Software disponível"
for p in cargo rustc gcc cc pkg-config caddy turnserver turnutils_uclient docker podman python3 nc ncat socat; do
  if tem "$p"; then printf '%-18s %s\n' "$p" "$(command -v "$p")"
  else printf '%-18s ausente\n' "$p"; fi
done
tem turnserver && rodar turnserver --version
tem cargo && rodar cargo --version
tem caddy && rodar caddy version

secao "Pacotes no repositório da distribuição"
if tem apt-cache; then
  rodar apt-cache policy coturn caddy
elif tem dnf; then
  rodar dnf -q info coturn caddy
elif tem zypper; then
  rodar zypper info coturn caddy
fi

secao "Fim"
echo "Avaliação concluída. Nenhuma configuração foi alterada."
