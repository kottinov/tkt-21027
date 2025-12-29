FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn run(_: std::net::TcpListener) -> Result<actix_web::dev::Server, std::io::Error> { unimplemented!() }" > src/lib.rs

RUN cargo build --release
RUN rm -rf src target/release/the-project target/release/deps/the_project* target/release/libthe_project* target/release/deps/libthe_project*

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -u 10001 app

COPY --from=builder /app/target/release/the-project /usr/local/bin/the-project

USER app

ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/the-project"]