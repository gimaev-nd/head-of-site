# Build stage: needs a C toolchain (rustls -> ring) and network for the
# standalone Tailwind CLI that the build script downloads.
FROM rust:1.98 AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY static ./static

RUN cargo build --release

# Runtime stage: the binary is self-contained (rustls bundles its root
# certificates), so a slim base image is enough.
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/head-of-site /usr/local/bin/head-of-site

ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000

CMD ["head-of-site"]
