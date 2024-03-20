FROM --platform=$BUILDPLATFORM rust:1.76.0-slim-buster@sha256:fa8fea738b02334822a242c8bf3faa47b9a98ae8ab587da58d6085ee890bbc33 as planner
WORKDIR /app
RUN cargo install cargo-chef --locked
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo chef prepare --recipe-path recipe.json

FROM --platform=$BUILDPLATFORM planner AS cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config=0.29-6 libssl-dev=1.1.1n-0+deb10u6 \
    && rm -rf /var/lib/apt/lists/*
RUN cargo chef cook --release --recipe-path recipe.json

FROM --platform=$BUILDPLATFORM rust:1.76.0-slim-buster@sha256:fa8fea738b02334822a242c8bf3faa47b9a98ae8ab587da58d6085ee890bbc33 AS builder
WORKDIR /app
COPY . .
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config=0.29-6 libssl-dev=1.1.1n-0+deb10u6 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release

FROM --platform=$BUILDPLATFORM gcr.io/distroless/cc-debian11
LABEL org.opencontainers.image.source=https://github.com/TidyBee/tidybee-agent
WORKDIR /app
COPY --from=builder /app/config /app/config
COPY --from=builder /app/tests/assets /app/tests/assets
COPY --from=builder /app/target/release/tidybee-agent /app/tidybee-agent
EXPOSE 8111
CMD ["/app/tidybee-agent"]
