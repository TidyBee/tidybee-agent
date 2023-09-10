FROM rust:1.72.0-alpine3.17@sha256:a51f8c7706159f07878e5c1d409c3e54a761145d5eba52fe200dd4f6d441c4fa
WORKDIR /app/
COPY src/ ./src/
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN apk add --no-cache build-base=0.5-r3
ENTRYPOINT ["cargo", "test"]
