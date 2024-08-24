FROM rust:1-alpine3.19 AS builder
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev
WORKDIR /pumpkin
COPY . /pumpkin
RUN cargo build --release
RUN strip target/release/pumpkin

FROM alpine:3.19
WORKDIR /pumpkin
RUN apk add --no-cache libgcc
COPY --from=builder /pumpkin/target/release/pumpkin /pumpkin/pumpkin
ENTRYPOINT ["/pumpkin/pumpkin"]
