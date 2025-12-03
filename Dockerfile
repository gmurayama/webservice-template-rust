FROM --platform=$BUILDPLATFORM rust:1.91.1-bookworm AS chef
WORKDIR /app
RUN apt update
RUN apt install build-essential gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross -y
RUN cargo install --locked cargo-chef@0.1.73
RUN rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json
 
FROM chef AS builder
ARG TARGETPLATFORM
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

COPY --from=planner /app/recipe.json recipe.json
COPY --from=planner /app/platform.sh platform.sh
RUN cargo chef cook \
  --release \
  --recipe-path recipe.json \
  --target $(sh platform.sh)
COPY . .
RUN cargo build -r \
		--target $(sh platform.sh) \
    --bin api
RUN mkdir /app/linux && \
  cp target/$(sh platform.sh)/release/api /app/${TARGETPLATFORM}
 
FROM debian:bookworm-slim AS runtime
WORKDIR /app
ARG TARGETPLATFORM
COPY --from=builder /app/${TARGETPLATFORM} /app/prog
CMD "/app/prog"
