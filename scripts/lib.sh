#!/bin/sh
#

WORK_DIR=${SCRIPT_DIR}/..
ARTIFACTS_DIR=${WORK_DIR}/artifacts
DOCKER_DIR=${WORK_DIR}/docker

mkdir -p ${ARTIFACTS_DIR}

get_yaab_version() {
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

get_yaab_major() {
  local cargo_file=$1
  if [ -f $cargo_file ]; then
    version=$(get_yaab_version $cargo_file)
    major=${version%%.*}
    echo $major
  else
    echo NA
  fi
}

get_yaab_minor() {
  local cargo_file=$1
  if [ -f $cargo_file ]; then
    version=$(get_yaab_version $cargo_file)
    minor=${version#*.}
    minor=${minor%%.*}
    echo $minor
  else
    echo NA
  fi
}

get_yaab_build() {
  local cargo_file=$1
  if [ -f $cargo_file ]; then
    version=$(get_yaab_version $cargo_file)
    build=${version##*.}
    echo $build
  else
    echo NA
  fi
}
