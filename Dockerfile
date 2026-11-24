# syntax=docker/dockerfile:1

FROM rust:1-slim-bookworm AS base
WORKDIR /src
COPY Cargo.toml rustfmt.toml ./
COPY src ./src

# Rust checks: `docker build --target ci .` (fails the build on any error)
FROM base AS ci
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    rustup component add rustfmt clippy \
    && cargo fmt --all -- --check \
    && cargo clippy --all-targets --all-features -- -D warnings \
    && cargo test --all-features

FROM base AS builder
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --bin dns2socks \
    && cp target/release/dns2socks /dns2socks

FROM debian:bookworm-slim
RUN useradd --system --no-create-home --shell /usr/sbin/nologin dns2socks
COPY --from=builder /dns2socks /usr/local/bin/dns2socks
USER dns2socks
EXPOSE 53/udp 53/tcp
ENTRYPOINT ["dns2socks"]
CMD ["--help"]
