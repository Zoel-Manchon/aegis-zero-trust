# syntax=docker/dockerfile:1
# =============================================================================
# aegis API — multi-stage build.
#
# PREREQUISITE: the backend uses compile-time sqlx macros (query!/query_as!),
# so the build is OFFLINE and needs a committed `.sqlx` cache. Generate it once
# against a running DB:   cargo sqlx prepare    (see DOCKER.md)
# =============================================================================

FROM rust:1-bookworm AS builder
WORKDIR /app
ENV SQLX_OFFLINE=true
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
# .dockerignore keeps target/, secrets, node_modules out of the context.
COPY . .
RUN cargo build --release --bin aegis

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates openssl curl jq \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/aegis /usr/local/bin/aegis
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh
ENV SERVER_ADDR=0.0.0.0:3000
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
