#!/usr/bin/env python3
"""Ouvinte temporário para testar entrada externa nas portas da implantação.

Escuta TCP 80/443 e UDP 3478/49160/49200 em IPv4 e IPv6 (socket dual-stack),
registra a origem de cada conexão/datagrama e responde "ok-<proto>-<porta>".
Não serve arquivos e encerra sozinho após DURACAO segundos.

Uso (como root, por causa das portas 80/443):
    python3 -I escutar-teste-portas.py
"""

import socket
import threading
import time

PORTAS_TCP = [80, 443]
PORTAS_UDP = [3478, 49160, 49200]
DURACAO = 15 * 60


def registrar(texto):
    print(time.strftime("%H:%M:%S"), texto, flush=True)


def criar(tipo, porta):
    s = socket.socket(socket.AF_INET6, tipo)
    s.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_V6ONLY, 0)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("::", porta))
    return s


def servir_tcp(porta):
    s = criar(socket.SOCK_STREAM, porta)
    s.listen()
    while True:
        conexao, origem = s.accept()
        registrar(f"TCP {porta} <- {origem[0]}")
        with conexao:
            conexao.sendall(f"ok-tcp-{porta}\n".encode())


def servir_udp(porta):
    s = criar(socket.SOCK_DGRAM, porta)
    while True:
        dados, origem = s.recvfrom(2048)
        registrar(f"UDP {porta} <- {origem[0]} ({len(dados)} B)")
        s.sendto(f"ok-udp-{porta}".encode(), origem)


for porta in PORTAS_TCP:
    threading.Thread(target=servir_tcp, args=(porta,), daemon=True).start()
for porta in PORTAS_UDP:
    threading.Thread(target=servir_udp, args=(porta,), daemon=True).start()

registrar(f"Escutando TCP {PORTAS_TCP} e UDP {PORTAS_UDP} por {DURACAO // 60} min. Ctrl+C encerra.")
try:
    time.sleep(DURACAO)
except KeyboardInterrupt:
    pass
registrar("Encerrado.")
