#!/bin/sh
#
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. $SCRIPT_DIR/lib.sh

VERSION=$(get_bakery_version ${WORK_DIR}/Cargo.toml)

echo "INFO: build bakery v${VERSION}"
(cd ${WORK_DIR}; cargo build --release)
cp ${WORK_DIR}/target/release/bakery ${ARTIFACTS_DIR}

echo "INFO: create bakery deb packagage"
(cd ${WORK_DIR}; ./scripts/do_deb_package.sh)

echo "INFO: build docker bakery-workspace"
(cd ${WORK_DIR}; ./docker/do_docker_build.sh)

