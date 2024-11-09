# Stage 1: Build
FROM rust:1-alpine3.20 AS builder
ENV RUSTFLAGS="-C target-feature=-crt-static -C target-cpu=native"
RUN apk add --no-cache musl-dev
WORKDIR /pumpkin
COPY . /pumpkin
RUN --mount=type=cache,sharing=private,target=/pumpkin/target \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release && cp target/release/pumpkin ./pumpkin.release
RUN strip pumpkin.release

# Stage 2: Final Image
FROM alpine:3.20
WORKDIR /pumpkin
RUN apk add --no-cache libgcc

# Copy the compiled binary from the builder stage
COPY --from=builder /pumpkin/pumpkin.release /pumpkin/pumpkin

ENV RUST_BACKTRACE=1
EXPOSE 25565

# Set the binary as the entrypoint
ENTRYPOINT ["/pumpkin/pumpkin"]
