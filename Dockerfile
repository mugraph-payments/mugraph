FROM docker.io/library/rust:1.92-slim AS builder

RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  protobuf-compiler \
  curl \
  wget \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY . /build

RUN cargo build --release --bin mugraph-node && \
  cp target/release/mugraph-node /tmp/mugraph-node

FROM docker.io/library/debian:bookworm-slim

RUN apt-get update && apt-get install -y \
  ca-certificates \
  libssl3 \
  && rm -rf /var/lib/apt/lists/* \
  && update-ca-certificates
RUN useradd -r -s /bin/false -m mugraph

WORKDIR /app
RUN mkdir -p /app/data && \
  chown -R mugraph:mugraph /app

# Copy the compiled validator artifacts
COPY --from=builder /build/validator/build /app/validator/build

COPY --from=builder /tmp/mugraph-node /app/mugraph-node
RUN chmod +x /app/mugraph-node && \
  chown mugraph:mugraph /app/mugraph-node
USER mugraph
ENV RUST_LOG=info
EXPOSE 9999
ENTRYPOINT ["/app/mugraph-node"]
CMD ["--addr", "0.0.0.0:9999"]
