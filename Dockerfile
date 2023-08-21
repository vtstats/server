# NOTE: use the same version of debian in both builder and runtime
FROM lukemathwalker/cargo-chef:0.1.60-rust-1.69.0-slim-bullseye AS chef
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

FROM gcr.io/distroless/cc-debian11
WORKDIR app
COPY --from=builder /app/target/release/vtstats vtstats
ENTRYPOINT ["./vtstats"]
