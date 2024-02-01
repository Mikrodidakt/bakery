#!/bin/sh
#
set -ex

VERSION=$1
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORK_DIR=$(mktemp -d --suffix=-bakery-deb)

mkdir -p ${WORK_DIR}/bakery
WORK_DIR=${WORK_DIR}/bakery
mkdir -p ${WORK_DIR}/usr/bin
cp ${SCRIPT_DIR}/../target/release/bakery ${WORK_DIR}/usr/bin/

mkdir -p ${WORK_DIR}/DEBIAN
touch ${WORK_DIR}/DEBIAN/control
cat <<EOT >> ${WORK_DIR}/DEBIAN/control
Package: bakery
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: all
Maintainer: Mans <mans.zigher@mikro.io>
Description: Build engine for the Yocto/OE
EOT

dpkg-deb --root-owner-group --build ${WORK_DIR}

cp ${WORK_DIR}/../bakery.deb ${SCRIPT_DIR}/../bakery-${VERSION}.deb


