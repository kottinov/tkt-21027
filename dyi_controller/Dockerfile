FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/dummysite-controller target/release/deps/dummysite_controller*

COPY src ./src

RUN cargo build --release

FROM debian:sid-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -u 10001 app

COPY --from=builder /app/target/release/dummysite-controller /usr/local/bin/dummysite-controller

USER app

ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/dummysite-controller"]
