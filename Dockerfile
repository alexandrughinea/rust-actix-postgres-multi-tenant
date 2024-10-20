FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y lld clang libssl-dev pkg-config

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin rust-actix-postgres-multi-tenant

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust-actix-postgres-multi-tenant rust-actix-postgres-multi-tenant
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENV APP_DEBUG=false

ENTRYPOINT ["./rust-actix-postgres-multi-tenant"]
