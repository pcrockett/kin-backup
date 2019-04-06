#!/bin/bash

# https://vaneyckt.io/posts/safer_bash_scripts_with_set_euxo_pipefail/
set -Eeuo pipefail

SCRIPT_DIR=`dirname "$(readlink -f "$0")"`
ROOT_DIR=`dirname "$SCRIPT_DIR"`
STAGING_DIR="$ROOT_DIR/kin_decrypt/all_executables"
DEST_ZIP="$ROOT_DIR/kin/src/compile/decrypt_executables.zip"

pushd "$ROOT_DIR" > /dev/null
cargo build --release --package kin_decrypt
cp target/release/decrypt "$STAGING_DIR/decrypt-linux"

if [ -f $DEST_ZIP ]
then
    rm $DEST_ZIP
fi

pushd "$STAGING_DIR" > /dev/null
zip "$DEST_ZIP" ./* --exclude .gitignore README.md
popd > /dev/null

# Now we're in the root of the repo. One more pop to get us back to wherever
# the user originally was when they first ran the script.
popd > /dev/null

