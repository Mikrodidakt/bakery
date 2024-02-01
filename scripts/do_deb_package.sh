#!/bin/sh
#
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. $SCRIPT_DIR/lib.sh

VERSION=$(get_bakery_version ${WORK_DIR}/Cargo.toml)
TEMP_WORK_DIR=$(mktemp -d --suffix=-bakery-deb)

mkdir -p ${TEMP_WORK_DIR}/bakery
TEMP_WORK_DIR=${TEMP_WORK_DIR}/bakery
mkdir -p ${TEMP_WORK_DIR}/usr/bin
cp ${ARTIFACTS_DIR}/bakery ${TEMP_WORK_DIR}/usr/bin/

mkdir -p ${TEMP_WORK_DIR}/DEBIAN
touch ${TEMP_WORK_DIR}/DEBIAN/control
cat <<EOT >> ${TEMP_WORK_DIR}/DEBIAN/control
Package: bakery
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: all
Maintainer: Mans <mans.zigher@mikro.io>
Description: Build engine for the Yocto/OE
EOT

dpkg-deb --root-owner-group --build ${TEMP_WORK_DIR}

cp ${TEMP_WORK_DIR}/../bakery.deb ${ARTIFACTS_DIR}/bakery-${VERSION}.deb


