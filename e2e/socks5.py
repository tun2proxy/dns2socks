#!/usr/bin/env python3
"""Minimal SOCKS5 server (CONNECT + UDP ASSOCIATE, optional user/pass auth).

Logs every handled request (CONNECT / UDP_ASSOCIATE / UDP_FORWARD / AUTH_OK /
AUTH_FAIL) so tests can prove traffic really went through the proxy.
"""
import argparse
import asyncio
import socket
import struct


async def read_addr(reader, atyp):
    if atyp == 1:
        host = socket.inet_ntoa(await reader.readexactly(4))
    elif atyp == 3:
        n = (await reader.readexactly(1))[0]
        host = (await reader.readexactly(n)).decode()
    elif atyp == 4:
        host = socket.inet_ntop(socket.AF_INET6, await reader.readexactly(16))
    else:
        raise ValueError(f"bad atyp {atyp}")
    (port,) = struct.unpack("!H", await reader.readexactly(2))
    return host, port


def parse_udp(data):
    # RSV(2) FRAG(1) ATYP(1) ADDR PORT DATA
    if len(data) < 10 or data[2] != 0:
        return None
    atyp = data[3]
    if atyp == 1:
        host = socket.inet_ntoa(data[4:8])
        off = 8
    elif atyp == 4:
        host = socket.inet_ntop(socket.AF_INET6, data[4:20])
        off = 20
    elif atyp == 3:
        n = data[4]
        host = data[5 : 5 + n].decode()
        off = 5 + n
    else:
        return None
    (port,) = struct.unpack("!H", data[off : off + 2])
    return host, port, data[off + 2 :]


def udp_header(host, port):
    try:
        return b"\x00\x00\x00\x01" + socket.inet_aton(host) + struct.pack("!H", port)
    except OSError:
        return b"\x00\x00\x00\x04" + socket.inet_pton(socket.AF_INET6, host) + struct.pack("!H", port)


class Relay(asyncio.DatagramProtocol):
    def connection_made(self, transport):
        self.transport = transport

    def datagram_received(self, data, addr):
        p = parse_udp(data)
        if p is None:
            return
        host, port, payload = p
        asyncio.ensure_future(self.forward(host, port, payload, addr))

    async def forward(self, host, port, payload, client):
        print(f"UDP_FORWARD {host}:{port}", flush=True)
        loop = asyncio.get_running_loop()
        fut = loop.create_future()

        class Up(asyncio.DatagramProtocol):
            def datagram_received(self, d, _a):
                if not fut.done():
                    fut.set_result(d)

        t, _ = await loop.create_datagram_endpoint(Up, remote_addr=(host, port))
        try:
            t.sendto(payload)
            resp = await asyncio.wait_for(fut, 5)
            self.transport.sendto(udp_header(host, port) + resp, client)
        except asyncio.TimeoutError:
            print(f"UDP_TIMEOUT {host}:{port}", flush=True)
        finally:
            t.close()


async def pipe(r, w):
    try:
        while data := await r.read(65536):
            w.write(data)
            await w.drain()
    except ConnectionError:
        pass
    finally:
        w.close()


async def handle(args, reader, writer):
    try:
        ver, nm = await reader.readexactly(2)
        methods = await reader.readexactly(nm)
        if ver != 5:
            return
        need_auth = args.user is not None
        want = 2 if need_auth else 0
        if want not in methods:
            writer.write(b"\x05\xff")
            await writer.drain()
            return
        writer.write(bytes([5, want]))
        await writer.drain()
        if need_auth:
            await reader.readexactly(1)
            u = (await reader.readexactly((await reader.readexactly(1))[0])).decode()
            p = (await reader.readexactly((await reader.readexactly(1))[0])).decode()
            if (u, p) != (args.user, args.password):
                print(f"AUTH_FAIL user={u}", flush=True)
                writer.write(b"\x01\x01")
                await writer.drain()
                return
            print(f"AUTH_OK user={u}", flush=True)
            writer.write(b"\x01\x00")
            await writer.drain()
        ver, cmd, _rsv, atyp = await reader.readexactly(4)
        host, port = await read_addr(reader, atyp)
        local_ip = writer.get_extra_info("sockname")[0]
        if cmd == 1:
            print(f"CONNECT {host}:{port}", flush=True)
            try:
                ur, uw = await asyncio.wait_for(asyncio.open_connection(host, port), 5)
            except Exception:  # noqa: BLE001
                writer.write(b"\x05\x05\x00\x01\x00\x00\x00\x00\x00\x00")
                await writer.drain()
                return
            writer.write(b"\x05\x00\x00\x01" + socket.inet_aton("0.0.0.0") + b"\x00\x00")
            await writer.drain()
            await asyncio.gather(pipe(reader, uw), pipe(ur, writer))
        elif cmd == 3:
            print("UDP_ASSOCIATE", flush=True)
            loop = asyncio.get_running_loop()
            transport, _ = await loop.create_datagram_endpoint(Relay, local_addr=(local_ip, 0))
            bport = transport.get_extra_info("sockname")[1]
            writer.write(b"\x05\x00\x00\x01" + socket.inet_aton(local_ip) + struct.pack("!H", bport))
            await writer.drain()
            try:
                await reader.read()  # association lives as long as the TCP connection
            finally:
                transport.close()
        else:
            writer.write(b"\x05\x07\x00\x01\x00\x00\x00\x00\x00\x00")
            await writer.drain()
    except (asyncio.IncompleteReadError, ConnectionError):
        pass
    finally:
        writer.close()


async def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bind", default="0.0.0.0")
    ap.add_argument("--port", type=int, default=1080)
    ap.add_argument("--user")
    ap.add_argument("--password")
    a = ap.parse_args()
    server = await asyncio.start_server(lambda r, w: handle(a, r, w), a.bind, a.port)
    print(f"socks5 listening on {a.bind}:{a.port} auth={'yes' if a.user else 'no'}", flush=True)
    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    asyncio.run(main())
