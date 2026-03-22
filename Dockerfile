# syntax=docker/dockerfile:1.6
# =============================================================================
# Tuitbot multi-stage Dockerfile
#
# Stage 1 (builder): compiles tuitbot-server and tuitbot-cli with cargo
# Stage 2 (runtime): minimal Debian slim image with only the produced binaries
#
# Why multi-stage: keeps the final image small (~80 MB vs ~2 GB for a full
# Rust toolchain image) and avoids shipping compiler artefacts to production.
#
# Build args:
#   BINARY — which binary to embed: "tuitbot-server" (default) or "tuitbot"
#             (the CLI).  Override at build time:
#             docker build --build-arg BINARY=tuitbot ...
# =============================================================================

FROM rust:1.85-slim-bookworm AS builder

# Install C toolchain and SQLite dev headers (required by sqlx + libsqlite3-sys).
# pkg-config is needed by the openssl-sys build script on some targets.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy dependency manifests first so Docker layer-caches them separately from
# source code.  This means `cargo fetch` only re-runs when Cargo.toml /
# Cargo.lock actually changes.
COPY Cargo.toml Cargo.lock ./
COPY crates/tuitbot-core/Cargo.toml     crates/tuitbot-core/Cargo.toml
COPY crates/tuitbot-server/Cargo.toml   crates/tuitbot-server/Cargo.toml
COPY crates/tuitbot-cli/Cargo.toml      crates/tuitbot-cli/Cargo.toml
COPY crates/tuitbot-mcp/Cargo.toml      crates/tuitbot-mcp/Cargo.toml

# Pre-fetch all dependencies (network is available in the builder stage only).
RUN cargo fetch

# Now copy actual source so incremental builds above are maximally cached.
COPY crates/ crates/
COPY migrations/ migrations/

# Build both server and CLI binaries in release mode.
# The tuitbot-server is the primary container entrypoint; tuitbot (CLI)
# is included so users can exec into the container for one-shot commands.
# sqlx in this codebase uses runtime queries only (no compile-time macros),
# so no DATABASE_URL is needed at build time.
RUN cargo build --release \
    --bin tuitbot-server \
    --bin tuitbot

# =============================================================================
# Runtime image — tuitbot-server (default) or tuitbot CLI
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Runtime deps only:
#   ca-certificates — HTTPS requests to X API / LLM providers
#   libsqlite3-0    — SQLite shared library (sqlx links against it dynamically)
#   curl            — used by docker-compose healthcheck to probe /health
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user to run the binary.
# UID/GID 1000 matches the default user on many Linux hosts, making volume
# mounts (e.g., ~/.tuitbot/) easier to own without chmod.
RUN useradd --uid 1000 --gid 0 --shell /bin/sh --create-home tuitbot

# Copy the selected binary from the builder stage.
COPY --from=builder /build/target/release/tuitbot-server /usr/local/bin/tuitbot-server
COPY --from=builder /build/target/release/tuitbot       /usr/local/bin/tuitbot

# Also include the example config so users can volume-mount a customised copy.
COPY config.example.toml /etc/tuitbot/config.example.toml

# Data directory for the tuitbot config file, SQLite DB, and API token.
# Map a host directory here to persist state between container restarts:
#   docker run -v ~/.tuitbot:/data ...
VOLUME ["/data"]

USER tuitbot

# The server reads TUITBOT_CONFIG (or the --config flag) to find config.toml.
# Default: /data/config.toml inside the container.
ENV TUITBOT_CONFIG=/data/config.toml \
    RUST_LOG=info

# Expose the default API server port.
EXPOSE 3001

# Default entrypoint runs tuitbot-server bound to all interfaces so Docker
# port-mapping works out of the box.  Override CMD for one-shot CLI usage:
#   docker run ghcr.io/aramirez087/tuitbot tuitbot --help
ENTRYPOINT ["/usr/local/bin/tuitbot-server"]
CMD ["--host", "0.0.0.0", "--config", "/data/config.toml"]
