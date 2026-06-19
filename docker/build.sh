#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

IMAGE="${LIANLI_BUILDER_IMAGE:-lianli-linux-builder}"
UID_HOST="$(id -u)"
GID_HOST="$(id -g)"

docker build \
    -f docker/build.Dockerfile \
    -t "$IMAGE" \
    --build-arg "USER_ID=$UID_HOST" \
    --build-arg "GROUP_ID=$GID_HOST" \
    .

mkdir -p .cache/cargo-registry .cache/cargo-git target

docker run --rm -it \
    -v "$PWD:/work" \
    -v "$PWD/.cache/cargo-registry:/home/builder/.cargo/registry" \
    -v "$PWD/.cache/cargo-git:/home/builder/.cargo/git" \
    "$IMAGE" "$@"

echo
echo "Build artifacts: $PWD/target/release"
