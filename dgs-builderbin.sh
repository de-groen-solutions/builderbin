#!/bin/bash

set -e

bold() {
    echo -e "\033[1m$1\033[0m"
}

yellow() {
    echo -e "\033[33m$1\033[0m"
}

dark_grey() {
    echo -e "\033[90m$1\033[0m"
}

indented() {
    while read -r line; do
		if [[ "$line" == *"ERROR"* ]] || [[ "$line" == *"error:"* ]]; then
			exit 1
		fi
        echo "${IMAGE_PARTIAL_NAME} | $(dark_grey "${line}")"
    done
}

CONTEXT_DIR=${CONTEXT_DIR:-${PWD}}
WORK_DIR=${WORK_DIR:-${PWD}}
IMAGE_PARTIAL_NAME=$1
shift || true
ENV_VARS=()
while [[ "$#" -gt 0 && "$1" == *"="* ]]; do
    ENV_VARS+=("--env '$1'")
    shift
done
RUN_ARGS=("$@")

BUILDERBIN_ENV=${BUILDERBIN_ENV:-}
echo check
# shellcheck disable=SC2016
if [ -z "$IMAGE_PARTIAL_NAME" ] || [ -z "$CONTEXT_DIR" ] || [ -z "$WORK_DIR" ] || [ -z "${RUN_ARGS[*]}" ]; then
    echo "Usage: $0 <IMAGE_PARTIAL_NAME> [ENV_VARS] <RUN_ARGS>"
    echo "Example: $0 aarch64 VAR1=value1 VAR2=value2 gcc --version"
	echo ""
	echo "Local images:"
	docker images | grep builderbin | awk '{print " - " $1 ":" $2}' | sort | uniq | sed 's/ghcr.io\/de-groen-solutions\/builderbin-//g'
    exit 1
fi

IMAGE_NAME="ghcr.io/de-groen-solutions/builderbin-$IMAGE_PARTIAL_NAME"

make_cmd() {
    echo docker run --rm -it --volume "${CONTEXT_DIR}:${CONTEXT_DIR}" --volume "${HOME}/sccache:${HOME}/sccache" --env SCCACHE_CACHE_SIZE=100G --env SCCACHE_DIR="${HOME}/sccache/sccache" --workdir "${WORK_DIR}" --privileged ${ENV_VARS[*]} "${IMAGE_NAME}" ${RUN_ARGS[*]}
}

yellow "$(bold "BUILDERBIN:")"
yellow "  CONTEXT_DIR: $CONTEXT_DIR"
yellow "  WORK_DIR: $WORK_DIR"
yellow "  IMAGE_NAME: $IMAGE_NAME"
yellow "  ENV_VARS: ${ENV_VARS[*]}"
yellow "  RUN_ARGS: ${RUN_ARGS[*]}"
yellow ""
yellow "$ $(make_cmd)"
yellow ""

eval "$(make_cmd)" 2>&1 | indented
