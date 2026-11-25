#!/usr/bin/env python3
"""E2E DNS client: query name over UDP and/or TCP and assert on the outcome."""
import argparse
import random
import socket
import struct
import sys


def build_query(name):
    qid = random.randint(0, 0xFFFF)
    q = b"".join(bytes([len(l)]) + l.encode() for l in name.rstrip(".").split(".")) + b"\x00"
    return qid, struct.pack("!HHHHHH", qid, 0x0100, 1, 0, 0, 0) + q + struct.pack("!HH", 1, 1)


def check(resp, qid, ip):
    rid, flags, _qd, an, _ns, _ar = struct.unpack("!HHHHHH", resp[:12])
    assert rid == qid, f"id mismatch {rid} != {qid}"
    assert flags & 0xF == 0, f"rcode {flags & 0xF}"
    assert an >= 1, "no answers"
    assert resp[-4:] == socket.inet_aton(ip), f"unexpected address {socket.inet_ntoa(resp[-4:])}"


def udp(server, port, payload, timeout):
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
        s.settimeout(timeout)
        s.sendto(payload, (server, port))
        return s.recvfrom(4096)[0]


def tcp(server, port, payload, timeout):
    with socket.create_connection((server, port), timeout=timeout) as s:
        s.settimeout(timeout)
        s.sendall(struct.pack("!H", len(payload)) + payload)
        buf = b""
        while len(buf) < 2:
            c = s.recv(2 - len(buf))
            if not c:
                raise RuntimeError("closed before length")
            buf += c
        (n,) = struct.unpack("!H", buf)
        data = b""
        while len(data) < n:
            c = s.recv(n - len(data))
            if not c:
                raise RuntimeError("short body")
            data += c
        return data


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--server", required=True)
    ap.add_argument("--port", type=int, default=53)
    ap.add_argument("--name", default="e2e.example.test")
    ap.add_argument("--expect-ip", default="10.20.30.40")
    ap.add_argument("--modes", default="udp,tcp")
    ap.add_argument("--timeout", type=float, default=6.0)
    ap.add_argument("--expect-fail", action="store_true")
    a = ap.parse_args()

    failed = False
    for mode in a.modes.split(","):
        qid, payload = build_query(a.name)
        try:
            resp = (udp if mode == "udp" else tcp)(a.server, a.port, payload, a.timeout)
            check(resp, qid, a.expect_ip)
            ok, detail = True, "answer ok"
        except Exception as e:  # noqa: BLE001
            ok, detail = False, f"{type(e).__name__}: {e}"
        expected = not a.expect_fail
        verdict = "PASS" if ok == expected else "FAIL"
        print(f"[{verdict}] {mode} {a.name} via {a.server}:{a.port} -> {detail} (expect {'success' if expected else 'failure'})")
        failed |= ok != expected
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
