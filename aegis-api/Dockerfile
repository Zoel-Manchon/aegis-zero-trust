# syntax=docker/dockerfile:1
# =============================================================================
# aegis API — multi-stage build.
#
# PREREQUISITE: the backend uses compile-time sqlx macros (query!/query_as!), so
# the build is OFFLINE and needs the committed `.sqlx` cache. Regenerate it only
# when a query changes:  cargo sqlx prepare   (see DOCKER.md)
# =============================================================================

FROM rust:1-bookworm AS builder
WORKDIR /app
ENV SQLX_OFFLINE=true
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# .dockerignore keeps target/, secrets and node_modules out of the context.
COPY . .

# Cache mounts make a rebuild after a code change take seconds instead of
# minutes. The binaries are copied out of the cached target/ inside the same
# RUN, because a cache mount is not visible to later stages.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin aegis --bin attack_simulator --bin admin_terminal \
    && mkdir -p /out \
    && cp target/release/aegis target/release/attack_simulator target/release/admin_terminal /out/

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates openssl curl jq \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /out/aegis            /usr/local/bin/aegis
COPY --from=builder /out/attack_simulator /usr/local/bin/attack_simulator
COPY --from=builder /out/admin_terminal   /usr/local/bin/admin_terminal

# `sed` strips CR: a Windows checkout without .gitattributes yields a CRLF
# shebang, which the kernel rejects ("no such file or directory" on start).
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN sed -i 's/\r$//' /usr/local/bin/docker-entrypoint.sh \
    && chmod +x /usr/local/bin/docker-entrypoint.sh

ENV SERVER_ADDR=0.0.0.0:3000
EXPOSE 3000

# Any HTTP response means axum is serving; 404 is fine as a liveness signal.
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=6 \
    CMD curl -sS -o /dev/null http://127.0.0.1:3000/login || exit 1

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
