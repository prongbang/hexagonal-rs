# ---------- Build stage ----------
FROM rust:1.86-slim-bookworm AS builder

# use: rustls-tls instead of OpenSSL
# reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "http2", "json"] }
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Cache deps
COPY Cargo.toml Cargo.lock ./
# create dummy main.rs for build deps cache
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release --bin service --locked || true

# Build real
RUN rm -rf src
COPY src ./src
RUN cargo build --release --bin service --locked

# ---------- Runtime stage ----------
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/service /app/service

EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/app/service"]
