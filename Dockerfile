FROM rust:1-alpine3.20 AS builder
ENV RUSTFLAGS="-C target-feature=-crt-static -C target-cpu=native"
RUN apk add --no-cache musl-dev

WORKDIR /pumpkin
COPY . /pumpkin

# build release
RUN --mount=type=cache,sharing=private,target=/pumpkin/target \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release && cp target/release/pumpkin ./pumpkin.release

# strip debug symbols from binary
RUN strip pumpkin.release

FROM alpine:3.20

# Identifying information for registries like ghcr.io
LABEL org.opencontainers.image.source=https://github.com/Snowiiii/Pumpkin

RUN apk add --no-cache libgcc

COPY --from=builder /pumpkin/pumpkin.release /pumpkin/pumpkin

# set workdir to /config, this is required to influence the PWD environment variable
# it allows for bind mounting the server files without overwriting the pumpkin
# executable (without requiring an `docker cp`-ing the binary to the host folder)
WORKDIR /config

ENV RUST_BACKTRACE=1
EXPOSE 25565
ENTRYPOINT [ "/pumpkin/pumpkin" ]
