FROM rust:1-alpine3.21 AS builder
ARG GIT_VERSION=Docker
ENV GIT_VERSION=$GIT_VERSION
ENV RUSTFLAGS="-C target-feature=-crt-static"
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

FROM alpine:3.21

# Identifying information for registries like ghcr.io
LABEL org.opencontainers.image.source=https://github.com/Snowiiii/Pumpkin

RUN apk add --no-cache libgcc

COPY --from=builder /pumpkin/pumpkin.release /bin/pumpkin

# set workdir to /pumpkin, this is required to influence the PWD environment variable
# it allows for bind mounting the server files without overwriting the pumpkin
# executable (without requiring an `docker cp`-ing the binary to the host folder)
WORKDIR /pumpkin

ENV RUST_BACKTRACE=1
EXPOSE 25565
ENTRYPOINT [ "/bin/pumpkin" ]
