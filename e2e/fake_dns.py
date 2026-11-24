#!/usr/bin/env python3
"""Tiny authoritative-for-everything DNS server used as the E2E upstream.

Answers every A query with a fixed IP (NOERROR/no-data for other types),
over both UDP and TCP, and logs each query so tests can prove it was reached.
"""
import argparse
import asyncio
import socket
import struct


def build_response(query: bytes, ip: str) -> bytes:
    if len(query) < 12:
        raise ValueError("short query")
    qid = query[:2]
    off = 12
    while query[off] != 0:
        off += query[off] + 1
    qname_end = off + 1
    qtype, _qclass = struct.unpack("!HH", query[qname_end : qname_end + 4])
    question = query[12 : qname_end + 4]
    name = decode(query, 12)
    answers = b""
    ancount = 0
    if qtype == 1:
        answers = b"\xc0\x0c" + struct.pack("!HHIH", 1, 1, 60, 4) + socket.inet_aton(ip)
        ancount = 1
    print(f"DNS_QUERY {name} type={qtype}", flush=True)
    header = qid + struct.pack("!HHHHH", 0x8180, 1, ancount, 0, 0)
    return header + question + answers


def decode(msg: bytes, off: int) -> str:
    labels = []
    while msg[off] != 0:
        n = msg[off]
        labels.append(msg[off + 1 : off + 1 + n].decode("ascii", "replace"))
        off += n + 1
    return ".".join(labels)


class UdpProto(asyncio.DatagramProtocol):
    def __init__(self, ip):
        self.ip = ip

    def connection_made(self, transport):
        self.transport = transport

    def datagram_received(self, data, addr):
        try:
            self.transport.sendto(build_response(data, self.ip), addr)
        except Exception as e:  # noqa: BLE001
            print(f"bad udp query: {e}", flush=True)


async def handle_tcp(ip, reader, writer):
    try:
        while True:
            hdr = await reader.readexactly(2)
            (n,) = struct.unpack("!H", hdr)
            q = await reader.readexactly(n)
            r = build_response(q, ip)
            writer.write(struct.pack("!H", len(r)) + r)
            await writer.drain()
    except (asyncio.IncompleteReadError, ConnectionError):
        pass
    finally:
        writer.close()


async def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bind", default="0.0.0.0")
    ap.add_argument("--port", type=int, default=53)
    ap.add_argument("--ip", default="10.20.30.40")
    a = ap.parse_args()
    loop = asyncio.get_running_loop()
    await loop.create_datagram_endpoint(lambda: UdpProto(a.ip), local_addr=(a.bind, a.port))
    await asyncio.start_server(lambda r, w: handle_tcp(a.ip, r, w), a.bind, a.port)
    print(f"fake dns listening on {a.bind}:{a.port} answering {a.ip}", flush=True)
    await asyncio.Event().wait()


if __name__ == "__main__":
    asyncio.run(main())
