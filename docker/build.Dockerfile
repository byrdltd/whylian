FROM rust:1.94-trixie

ARG DEBIAN_FRONTEND=noninteractive
ARG USER_ID=1000
ARG GROUP_ID=1000

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl git \
    build-essential pkg-config \
    clang cmake ninja-build nasm \
    libssl-dev \
    libhidapi-dev libusb-1.0-0-dev libudev-dev libfontconfig-dev \
    libxkbcommon-dev libwayland-dev libx11-dev libinput-dev libdrm-dev \
    libgl-dev libegl-dev \
    libavcodec-dev libavformat-dev libswscale-dev libavutil-dev \
    ffmpeg \
  && rm -rf /var/lib/apt/lists/*

RUN groupadd -g ${GROUP_ID} builder \
  && useradd -m -u ${USER_ID} -g ${GROUP_ID} -s /bin/bash builder \
  && mkdir -p /build/target \
  && chown -R builder:builder /build

USER builder
WORKDIR /work

ENV CARGO_TARGET_DIR=/build/target

CMD ["bash", "-c", "\
  set -euo pipefail; \
  set -x; \
  command -v cargo && cargo -V; \
  cd /work && cargo build --release; \
  rm -rf /work/target/release; \
  mkdir -p /work/target; \
  cp -a /build/target/release /work/target/; \
"]
