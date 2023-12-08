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

# Put args into array RUN_ARGS
shift || true
ENV_VARS=()
while [[ "$#" -gt 0 && "$1" == *"="* ]]; do
    ENV_VARS+=("--env '$1'")
    shift
done
RUN_ARGS=("$@")

BUILDERBIN_TTY=${BUILDERBIN_TTY:-}
BUILDERBIN_ENV=${BUILDERBIN_ENV:-}

print_local_partial_image_names() {
	docker images | grep builderbin | sed 's/ghcr.io\/de-groen-solutions\/builderbin-//g' | awk '{print $1 ":" $2}' | sort | uniq
}

# shellcheck disable=SC2016
usage() {
    echo "Usage:"
	echo "  $0 <IMAGE_PARTIAL_NAME> [ENV_VARS] <RUN_ARGS>"
	echo
    echo "Example:"
	echo "  $0 aarch64 VAR1=value1 VAR2=value2 gcc --version"
	echo
	echo "Local partial image names:"
	print_local_partial_image_names | awk '{print " - " $0}'
}

IMAGE_NAME="ghcr.io/de-groen-solutions/builderbin-$IMAGE_PARTIAL_NAME"

make_cmd() {
    echo docker run \
		--privileged --rm -it \
		--workdir "${WORK_DIR}" \
		--volume "${CONTEXT_DIR}:${CONTEXT_DIR}" \
		--volume "${HOME}/.cargo/registry:/root/.cargo/registry" \
		${ENV_VARS[*]} \
		${IMAGE_NAME} \
		${RUN_ARGS[*]}
}

main() {
	if [ -z "$IMAGE_PARTIAL_NAME" ] || [ -z "$CONTEXT_DIR" ] || [ -z "$WORK_DIR" ] || [ -z "${RUN_ARGS[*]}" ]; then
		usage
		exit 1
	fi

	yellow "$(bold "BUILDERBIN:")"
	yellow "  CONTEXT_DIR: $CONTEXT_DIR"
	yellow "  WORK_DIR: $WORK_DIR"
	yellow "  IMAGE_NAME: $IMAGE_NAME"
	yellow "  ENV_VARS: ${ENV_VARS[*]}"
	yellow "  RUN_ARGS: ${RUN_ARGS[*]}"
	yellow ""
	yellow "$ $(make_cmd)"
	yellow ""

	if [ $BUILDERBIN_TTY ]; then
		eval "$(make_cmd)"
		exit 0
	else
		eval "$(make_cmd)" 2>&1 | indented
	fi
}

main

