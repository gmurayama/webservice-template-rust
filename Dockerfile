FROM --platform=$BUILDPLATFORM rust:1.82.0-alpine AS chef
WORKDIR /app
ENV PKGCONFIG_SYSROOTDIR=/
RUN apk update
RUN apk add --no-cache alpine-sdk openssl-dev zig
RUN cargo install --locked cargo-zigbuild cargo-chef
RUN rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
 
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json
 
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --release --zigbuild \
  --target x86_64-unknown-linux-gnu --target aarch64-unknown-linux-gnu
 
COPY . .
RUN cargo zigbuild -r \
    --target x86_64-unknown-linux-gnu --target aarch64-unknown-linux-gnu \
    --bin api
RUN mkdir /app/linux && \
  cp target/aarch64-unknown-linux-gnu/release/api /app/linux/arm64 && \
  cp target/x86_64-unknown-linux-gnu/release/api /app/linux/amd64
 
FROM debian:bullseye-slim AS runtime
WORKDIR /app
ARG TARGETPLATFORM
COPY --from=builder /app/${TARGETPLATFORM} /app/prog
CMD "/app/prog"
