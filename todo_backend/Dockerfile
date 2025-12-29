FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn run(_: std::net::TcpListener) -> Result<actix_web::dev::Server, std::io::Error> { unimplemented!() }" > src/lib.rs

RUN cargo build --release
RUN rm -rf src target/release/todo-backend target/release/deps/todo_backend* target/release/libtodo_backend* target/release/deps/libtodo_backend*

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -u 10001 app

COPY --from=builder /app/target/release/todo-backend /usr/local/bin/todo-backend

USER app

ENV PORT=3000
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/todo-backend"]
