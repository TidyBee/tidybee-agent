FROM rust:1.76.0-slim-buster@sha256:fa8fea738b02334822a242c8bf3faa47b9a98ae8ab587da58d6085ee890bbc33
WORKDIR /app/
RUN --mount=type=cache,target=/var/cache/apt \
    apt-get update \
    && apt-get install -y --no-install-recommends pkg-config=0.29-6 libssl-dev=1.1.1n-0+deb10u6 \
    && rm -rf /var/lib/apt/lists/*
EXPOSE 8111
COPY . .
RUN --mount=type=cache,target=/app/target/ cargo build --release
CMD ["./target/release/tidybee-agent"]
