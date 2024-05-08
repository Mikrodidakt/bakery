#!/bin/sh
#
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. $SCRIPT_DIR/lib.sh

CARGO_TOML_FILE="${WORK_DIR}/Cargo.toml"
VERSION=$(get_bakery_version ${CARGO_TOML_FILE})
MAJOR=$(get_bakery_major ${CARGO_TOML_FILE})
MINOR=$(get_bakery_minor ${CARGO_TOML_FILE})
BUILD=$(get_bakery_build ${CARGO_TOML_FILE})
INCBUILD=$((BUILD+1))

echo "INFO: current version ${MAJOR}.${MINOR}.${BUILD}"

sed -i "s/version = \"$VERSION\"/version = \"${MAJOR}.${MINOR}.${INCBUILD}\"/g" "${CARGO_TOML_FILE}"

echo "INFO: new version $(get_bakery_version ${CARGO_TOML_FILE})"
