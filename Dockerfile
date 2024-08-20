FROM rust:1.79 AS builder
WORKDIR /usr/src/pumpkin
COPY . .
RUN ls
RUN cargo install --path ./pumpkin

FROM rust
WORKDIR /pumpkin
RUN apt update && apt-get install -y libssl-dev pkg-config ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/pumpkin /pumpkin/pumpkin
CMD ["/pumpkin/pumpkin"]

# FROM rust:1.79-alpine AS builder
# WORKDIR /usr/src/pumpkin
# COPY . .
# RUN apk add openssl-dev libssl3 ca-certificates pkgconfig musl-dev
# RUN cargo install --path ./pumpkin

# FROM rust:1.79-alpine
# WORKDIR /pumpkin
# RUN apk add openssl ca-certificates pkgconfig
# COPY --from=builder /usr/local/cargo/bin/pumpkin /pumpkin/pumpkin
# CMD ["/pumpkin/pumpkin"]

#docker run --rm -v "./world:/pumpkin/world" pumpkin
