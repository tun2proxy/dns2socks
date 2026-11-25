#!/usr/bin/env bash
# Real end-to-end test, fully inside Docker (works with a remote Docker daemon).
#
#   client --DNS--> dns2socks --SOCKS5--> proxy --DNS--> upstream DNS
#
# The client+dns2socks network and the upstream network are separate and
# `--internal`, so dns2socks can only reach the upstream *through* the proxy.
#
# Requires images: $APP_IMAGE (the real dns2socks image) and $TOOLS_IMAGE (e2e/Dockerfile).
set -euo pipefail

APP_IMAGE=${APP_IMAGE:-dns2socks:e2e}
TOOLS_IMAGE=${TOOLS_IMAGE:-dns2socks-e2e-tools:latest}
EXPECT_IP=10.20.30.40
P="e2e-${GITHUB_RUN_ID:-local}-$$"
CLIENT_NET="$P-client"
UPSTREAM_NET="$P-upstream"
LABEL="e2e.run=$P"
FAILURES=0

remove_containers() {
  local ids
  ids=$(docker ps -aq --filter "label=$LABEL")
  [ -z "$ids" ] || docker rm -f $ids >/dev/null 2>&1 || true
}

cleanup() {
  local rc=$?
  if [ "$rc" -ne 0 ]; then dump_logs; fi
  remove_containers
  docker network rm "$CLIENT_NET" "$UPSTREAM_NET" >/dev/null 2>&1 || true
  exit "$rc"
}
trap cleanup EXIT

dump_logs() {
  local c
  for c in dns proxy app; do
    echo "----- logs: $c -----"
    docker logs "$P-$c" 2>&1 | tail -n 40 || true
  done
}

wait_log() { # container pattern [timeout_s]
  local c=$1 pat=$2 t=${3:-30} i
  for ((i = 0; i < t * 2; i++)); do
    if docker logs "$c" 2>&1 | grep -q -- "$pat"; then return 0; fi
    if [ "$(docker inspect -f '{{.State.Running}}' "$c" 2>/dev/null)" != "true" ]; then
      echo "container $c exited early" >&2
      return 1
    fi
    sleep 0.5
  done
  echo "timeout waiting for '$pat' in $c" >&2
  return 1
}

log_has() { docker logs "$P-$1" 2>&1 | grep -q -- "$2"; }

assert_log() { # container pattern
  if log_has "$1" "$2"; then echo "  ok   $1 log has '$2'"; else echo "  FAIL $1 log lacks '$2'"; FAILURES=$((FAILURES + 1)); fi
}
assert_no_log() { # container pattern
  if log_has "$1" "$2"; then echo "  FAIL $1 log unexpectedly has '$2'"; FAILURES=$((FAILURES + 1)); else echo "  ok   $1 log lacks '$2'"; fi
}

start_dns() {
  docker run -d --name "$P-dns" --label "$LABEL" --network "$UPSTREAM_NET" "$TOOLS_IMAGE" \
    python fake_dns.py --port 53 --ip "$EXPECT_IP" >/dev/null
  wait_log "$P-dns" "fake dns listening"
  UP_IP=$(docker inspect -f "{{(index .NetworkSettings.Networks \"$UPSTREAM_NET\").IPAddress}}" "$P-dns")
}

start_proxy() { # [user password]
  local extra=()
  if [ $# -eq 2 ]; then extra=(--user "$1" --password "$2"); fi
  docker run -d --name "$P-proxy" --label "$LABEL" --network "$CLIENT_NET" "$TOOLS_IMAGE" \
    python socks5.py --port 1080 "${extra[@]}" >/dev/null
  docker network connect "$UPSTREAM_NET" "$P-proxy"
  wait_log "$P-proxy" "socks5 listening"
}

start_app() { # socks5-url, extra dns2socks args...
  local url=$1
  shift
  docker run -d --name "$P-app" --label "$LABEL" --network "$CLIENT_NET" "$APP_IMAGE" \
    -l 0.0.0.0:53 -d "$UP_IP:53" -s "$url" -t 3 "$@" >/dev/null
  wait_log "$P-app" "Udp listening on"
  wait_log "$P-app" "TCP listening on"
}

query() { # extra client args...
  docker run --rm --network "$CLIENT_NET" "$TOOLS_IMAGE" \
    python client.py --server "$P-app" --port 53 --expect-ip "$EXPECT_IP" "$@" || FAILURES=$((FAILURES + 1))
}

scenario() {
  echo
  echo "=== scenario: $1 ==="
  remove_containers
}

docker network create --internal --label "$LABEL" "$CLIENT_NET" >/dev/null
docker network create --internal --label "$LABEL" "$UPSTREAM_NET" >/dev/null

scenario "UDP mode (default): UDP ASSOCIATE for UDP queries, CONNECT for TCP queries"
start_dns; start_proxy
start_app "socks5://$P-proxy:1080"
query
assert_log proxy "UDP_ASSOCIATE"
assert_log proxy "UDP_FORWARD $UP_IP:53"
assert_log proxy "CONNECT $UP_IP:53"
assert_log dns "DNS_QUERY e2e.example.test"

scenario "--force-tcp: everything goes through SOCKS5 CONNECT"
start_dns; start_proxy
start_app "socks5://$P-proxy:1080" -f
query
assert_log proxy "CONNECT $UP_IP:53"
assert_no_log proxy "UDP_ASSOCIATE"
assert_log dns "DNS_QUERY e2e.example.test"

scenario "SOCKS5 username/password auth (percent-encoded password)"
start_dns; start_proxy alice 'p@ss'
start_app "socks5://alice:p%40ss@$P-proxy:1080"
query
assert_log proxy "AUTH_OK user=alice"
assert_log dns "DNS_QUERY e2e.example.test"

scenario "wrong SOCKS5 password: queries must fail and never reach upstream"
start_dns; start_proxy alice 'p@ss'
start_app "socks5://alice:wrong@$P-proxy:1080"
query --expect-fail --timeout 5
assert_log proxy "AUTH_FAIL user=alice"
assert_no_log dns "DNS_QUERY"

scenario "--cache-records: second query answered after upstream is gone"
start_dns; start_proxy
start_app "socks5://$P-proxy:1080" -c -v trace
query
docker stop "$P-dns" >/dev/null
query
assert_log app "cache hit"

echo
if [ "$FAILURES" -ne 0 ]; then
  echo "E2E FAILED: $FAILURES check(s) failed"
  dump_logs
  exit 1
fi
echo "E2E PASSED"
