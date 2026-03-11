# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1.91-slim AS builder

WORKDIR /app

# Install build deps (openssl for aws-sdk TLS)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies — copy manifests first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Build the actual app
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM rust:1.91-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/volunteer_match_service ./
COPY --from=builder /app/migrations ./migrations

EXPOSE 8080

CMD ["./volunteer_match_service"]
