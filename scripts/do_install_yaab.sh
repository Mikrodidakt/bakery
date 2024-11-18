#!/bin/sh
#

YAAB_VERSION=$1
TEMP_WORK_DIR=$(mktemp -d --suffix=-yaab-deb)
(cd ${TEMP_WORK_DIR}; wget https://github.com/Mikrodidakt/bakery/releases/download/v${YAAB_VERSION}/yaab-${YAAB_VERSION}.deb)
sudo dpkg -i ${TEMP_WORK_DIR}/yaab-${YAAB_VERSION}.deb
