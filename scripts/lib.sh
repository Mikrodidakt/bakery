#!/bin/sh
#

WORK_DIR=${SCRIPT_DIR}/..
ARTIFACTS_DIR=${WORK_DIR}/artifacts
DOCKER_DIR=${WORK_DIR}/docker

mkdir -p ${ARTIFACTS_DIR}

get_bakery_version() {
  local cargo_file=$1
  if [ -f $cargo_file ]; then
    version=$(grep version $cargo_file)
    version=$(echo $version | awk '{ print $3 }')
    version=${version#\"*}
    version=${version%*\"}
    echo $version
  else
    echo NA
  fi
}
