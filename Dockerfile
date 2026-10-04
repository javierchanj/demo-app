FROM rust:1.98.1-slim-bookworm AS builder

WORKDIR /usr/src/

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/app

COPY --from=builder /usr/src/assets assets
COPY --from=builder /usr/src/config config
COPY --from=builder /usr/src/target/release/demo_app-cli demo_app-cli

ENTRYPOINT ["/usr/app/demo_app-cli", "start"]
