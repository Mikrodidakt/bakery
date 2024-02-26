#!/bin/sh
#

BAKERY_VERSION=$1
TEMP_WORK_DIR=$(mktemp -d --suffix=-bakery-deb)
(cd ${TEMP_WORK_DIR}; wget https://github.com/Mikrodidakt/bakery/releases/download/v${BAKERY_VERSION}/bakery-${BAKERY_VERSION}.deb)
sudo dpkg -i ${TEMP_WORK_DIR}/bakery-${BAKERY_VERSION}.deb
