#!/bin/bash

# https://vaneyckt.io/posts/safer_bash_scripts_with_set_euxo_pipefail/
set -Eeuo pipefail

ROOT_DIR=`dirname "$(readlink -f "$0")"`
PREPARE_SCRIPT="$ROOT_DIR/scripts/prepare-decrypt-exes.sh"

$PREPARE_SCRIPT

pushd "$ROOT_DIR" > /dev/null
cargo build --release --package kin
popd > /dev/null
