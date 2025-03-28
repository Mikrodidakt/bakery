#!/bin/sh
#
set -e

VARIANT=$1

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. ${SCRIPT_DIR}/lib.sh
VERSION=$(get_bakery_version ${WORK_DIR}/Cargo.toml)
TEMP_WORK_DIR=$(mktemp -d --suffix=-bakery-deb)

if [ ! -n "${VARIANT}" ]; then
    VARIANT=glibc
fi

check_variant ${VARIANT}

mkdir -p ${TEMP_WORK_DIR}/bakery
TEMP_WORK_DIR=${TEMP_WORK_DIR}/bakery
mkdir -p ${TEMP_WORK_DIR}/usr/bin
mkdir -p ${TEMP_WORK_DIR}/etc/bakery
cp ${ARTIFACTS_DIR}/bakery ${TEMP_WORK_DIR}/usr/bin/
cp ${SCRIPT_DIR}/bakery.bashrc ${TEMP_WORK_DIR}/etc/bakery/bakery.bashrc

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

cp ${TEMP_WORK_DIR}/../bakery.deb ${ARTIFACTS_DIR}/bakery-x86_64-${VARIANT}-v${VERSION}.deb
(cd ${ARTIFACTS_DIR}; ln -sf bakery-x86_64-${VARIANT}-v${VERSION}.deb bakery.deb && ln -sf bakery-x86_64-${VARIANT}-v${VERSION}.deb bakery-x86_64-${VARIANT}.deb)

