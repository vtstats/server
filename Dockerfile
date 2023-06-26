FROM lukemathwalker/cargo-chef:0.1.61-rust-1.70-slim-bullseye AS chef
WORKDIR app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# web
FROM gcr.io/distroless/cc-debian11 AS web
WORKDIR app
COPY --from=builder /app/target/release/vtstat-web vtstat-web
CMD ["./vtstat-web"]

# worker
FROM gcr.io/distroless/cc-debian11 AS worker
WORKDIR app
COPY --from=builder /app/target/release/vtstat-worker vtstat-worker
CMD ["./vtstat-worker"]
