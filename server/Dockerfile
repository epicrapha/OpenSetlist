# Multi-stage build: release binary only in the final image.
FROM rust:1-bookworm AS builder
WORKDIR /app

# Dependency layer: manifest first so source edits don't refetch crates.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs \
    && cargo build --release --locked \
    && rm -rf src

# Real sources. `sqlx::migrate!` embeds the migrations at compile time.
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs src/lib.rs && cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/opensetlist-server /usr/local/bin/opensetlist-server

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
USER 10001

CMD ["opensetlist-server"]
