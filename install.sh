#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

echo "Installing dgs-builderbin..."
ln -s "${DIR}/dgs-builderbin.sh" "${HOME}/.local/bin/dgs-builderbin"

