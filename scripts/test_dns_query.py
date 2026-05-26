#!/usr/bin/env python3
"""Send a DNS query for baidu.com to a local resolver.

Defaults:
- host: 127.0.0.1
- port: 53
- name: baidu.com
- type: A
- transport: TCP

Examples:
  python3 scripts/test_dns_query.py
  python3 scripts/test_dns_query.py --udp
  python3 scripts/test_dns_query.py --name baidu.com --server 127.0.0.1 --port 53
"""

from __future__ import annotations

import argparse
import random
import socket
import struct
import sys
from typing import List, Tuple


def encode_name(name: str) -> bytes:
    labels = name.rstrip(".").split(".")
    encoded = bytearray()
    for label in labels:
        label_bytes = label.encode("ascii")
        if len(label_bytes) > 63:
            raise ValueError(f"label too long: {label}")
        encoded.append(len(label_bytes))
        encoded.extend(label_bytes)
    encoded.append(0)
    return bytes(encoded)


def decode_name(message: bytes, offset: int) -> Tuple[str, int]:
    labels: List[str] = []
    jumped = False
    original_offset = offset

    while True:
        length = message[offset]
        if length == 0:
            offset += 1
            break

        if length & 0xC0 == 0xC0:
            pointer = ((length & 0x3F) << 8) | message[offset + 1]
            if not jumped:
                original_offset = offset + 2
            offset = pointer
            jumped = True
            continue

        offset += 1
        labels.append(message[offset : offset + length].decode("ascii", errors="replace"))
        offset += length

    return ".".join(labels), (original_offset if jumped else offset)


def build_query(name: str, qtype: int = 1) -> Tuple[int, bytes]:
    query_id = random.randint(0, 0xFFFF)
    flags = 0x0100  # recursion desired
    header = struct.pack("!HHHHHH", query_id, flags, 1, 0, 0, 0)
    question = encode_name(name) + struct.pack("!HH", qtype, 1)
    return query_id, header + question


def parse_response(message: bytes, expected_id: int) -> None:
    if len(message) < 12:
        raise ValueError("response too short")

    (response_id, flags, qdcount, ancount, nscount, arcount) = struct.unpack("!HHHHHH", message[:12])
    if response_id != expected_id:
        raise ValueError(f"unexpected response id: {response_id} != {expected_id}")

    rcode = flags & 0x000F
    if rcode != 0:
        raise RuntimeError(f"dns error rcode={rcode}")

    offset = 12
    for _ in range(qdcount):
        _, offset = decode_name(message, offset)
        offset += 4

    answers = []
    for _ in range(ancount):
        name, offset = decode_name(message, offset)
        rtype, rclass, ttl, rdlength = struct.unpack("!HHIH", message[offset : offset + 10])
        offset += 10
        rdata = message[offset : offset + rdlength]
        offset += rdlength

        if rtype == 1 and rclass == 1 and rdlength == 4:
            ip = socket.inet_ntoa(rdata)
            answers.append((name, "A", ttl, ip))
        elif rtype == 28 and rclass == 1 and rdlength == 16:
            ip = socket.inet_ntop(socket.AF_INET6, rdata)
            answers.append((name, "AAAA", ttl, ip))
        else:
            answers.append((name, f"TYPE{rtype}", ttl, rdata.hex()))

    print(f"answers={ancount} ns={nscount} additional={arcount}")
    for name, record_type, ttl, value in answers:
        print(f"{name} {ttl} IN {record_type} {value}")


def send_udp(server: str, port: int, payload: bytes, timeout: float) -> bytes:
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
        sock.settimeout(timeout)
        sock.sendto(payload, (server, port))
        data, _ = sock.recvfrom(4096)
        return data


def send_tcp(server: str, port: int, payload: bytes, timeout: float) -> bytes:
    with socket.create_connection((server, port), timeout=timeout) as sock:
        sock.settimeout(timeout)
        sock.sendall(struct.pack("!H", len(payload)) + payload)
        header = sock.recv(2)
        if len(header) < 2:
            raise RuntimeError("short TCP DNS header")
        (length,) = struct.unpack("!H", header)
        data = bytearray()
        while len(data) < length:
            chunk = sock.recv(length - len(data))
            if not chunk:
                break
            data.extend(chunk)
        if len(data) != length:
            raise RuntimeError("short TCP DNS body")
        return bytes(data)


def main() -> int:
    parser = argparse.ArgumentParser(description="Query baidu.com through a local DNS server")
    parser.add_argument("--server", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=53)
    parser.add_argument("--name", default="baidu.com")
    parser.add_argument("--timeout", type=float, default=5.0)
    parser.add_argument("--udp", action="store_true", help="use UDP instead of TCP")
    args = parser.parse_args()

    query_id, payload = build_query(args.name)
    transport = "UDP" if args.udp else "TCP"
    print(f"query={args.name} server={args.server}:{args.port} transport={transport}")

    if args.udp:
        response = send_udp(args.server, args.port, payload, args.timeout)
    else:
        response = send_tcp(args.server, args.port, payload, args.timeout)

    parse_response(response, query_id)
    return 0


if __name__ == "__main__":
    sys.exit(main())
