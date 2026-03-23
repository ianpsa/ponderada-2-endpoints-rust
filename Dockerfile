FROM rust:1.88-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release --locked
RUN rm src/main.rs
COPY src ./src
RUN touch src/main.rs && cargo build --release --locked

FROM debian:bookworm-slim
WORKDIR /app
RUN mkdir /data
COPY --from=builder /app/target/release/ponderada-2-endpoints ./server
EXPOSE 8080
ENTRYPOINT ["./server"]
