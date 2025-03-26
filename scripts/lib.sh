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

get_bakery_major() {
  local cargo_file=$1
  if [ -f $cargo_file ]; then
    version=$(get_bakery_version $cargo_file)
    major=${version%%.*}
    echo $major
  else
    echo NA
  fi
}

get_bakery_minor() {
  local cargo_file=$1
  if [ -f $cargo_file ]; then
    version=$(get_bakery_version $cargo_file)
    minor=${version#*.}
    minor=${minor%%.*}
    echo $minor
  else
    echo NA
  fi
}

get_bakery_build() {
  local cargo_file=$1
  if [ -f $cargo_file ]; then
    version=$(get_bakery_version $cargo_file)
    build=${version##*.}
    echo $build
  else
    echo NA
  fi
}

check_variant() {
  local variant=$1
  if [ "$variant" != "glibc" ] && [ "$variant" != "musl" ]; then
    echo "ERROR: Invalid target '${variant}'. Accepted values are 'glibc' or 'musl'." >&2
    exit 1
  fi
}
