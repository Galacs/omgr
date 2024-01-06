FROM rust:1.74.0 AS chef 
# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 
# # RUN cargo install sqlx-cli
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR app
COPY . .
# COPY --from=planner /app/recipe.json recipe.json
# RUN cargo chef cook --release --recipe-path recipe.json
# Build application
# RUN sqlx database create
ENV SQLX_OFFLINE true
RUN cargo build --release --all


FROM debian:bookworm-slim AS omgr
WORKDIR app
RUN apt-get update && apt-get install -y libssl-dev ca-certificates
COPY --from=builder /app/app/target/release/omgr /usr/local/bin
ENTRYPOINT ["/usr/local/bin/omgr"]

FROM debian:bookworm-slim AS cron
WORKDIR app
RUN apt-get update && apt-get install -y libssl-dev ca-certificates cron
COPY --from=builder /app/app/target/release/cron /usr/local/bin
RUN crontab -l | { cat; echo "*/5 * * * * /usr/local/bin/cron"; } | crontab -
ENTRYPOINT ["/usr/sbin/cron"]
CMD ["-f", "-l", "2", "-L", "/dev/stdout"]

FROM debian:bookworm-slim AS web
WORKDIR app
RUN apt-get update && apt-get install -y libssl-dev ca-certificates
COPY --from=builder /app/app/target/release/web /usr/local/bin
ENTRYPOINT ["/usr/local/bin/web"]