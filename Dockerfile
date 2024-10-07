FROM rust:1-alpine3.20 AS builder
ENV RUSTFLAGS="-C target-feature=-crt-static -C target-cpu=native"
RUN apk add --no-cache musl-dev
WORKDIR /pumpkin
COPY . /pumpkin
RUN cargo build --release
RUN strip target/release/pumpkin

FROM alpine:3.20
WORKDIR /pumpkin
RUN apk add --no-cache libgcc
COPY --from=builder /pumpkin/target/release/pumpkin /pumpkin/pumpkin
ENTRYPOINT ["/pumpkin/pumpkin"]
