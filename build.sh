#!/bin/bash

set -e

(cd transpiler && cargo run -- ../preset.yaml)

docker buildx build \
    --file ./dockerfiles/ghcr-io-de-groen-solutions-builderbin-aarch64-base-18-04.Dockerfile \
    --tag ghcr.io/de-groen-solutions/builderbin-aarch64-base:18.04 \
    --load . || exit 99

docker buildx build \
    --file ./dockerfiles/ghcr-io-de-groen-solutions-builderbin-aarch64-gcc-18-04.Dockerfile \
    --tag ghcr.io/de-groen-solutions/builderbin-aarch64-gcc:18.04 \
    --load . || exit 99

docker buildx build \
    --file ./dockerfiles/ghcr-io-de-groen-solutions-builderbin-aarch64-rust-18-04.Dockerfile \
    --tag ghcr.io/de-groen-solutions/builderbin-aarch64-rust:18.04 \
    --load . || exit 99

docker buildx build \
    --file ./dockerfiles/ghcr-io-de-groen-solutions-builderbin-aarch64-agentui-18-04.Dockerfile \
    --tag ghcr.io/de-groen-solutions/builderbin-aarch64-agentui:18.04 \
    --load . || exit 99

echo "Successfully built all images"
