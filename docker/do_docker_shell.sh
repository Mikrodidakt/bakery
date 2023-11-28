#!/bin/sh
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. $SCRIPT_DIR/lib_docker.sh
DOCKER_REGISTRY=$(get_docker_registry ${SETTINGS_FILE})
DOCKER_IMAGE=$(get_docker_image ${SETTINGS_FILE})
DOCKER_TAG=$(get_docker_tag ${SETTINGS_FILE})
DOCKER_ARGS="-it $(get_docker_args ${SETTINGS_FILE})"
CMD="$@"

if [ "x$CMD" = "x" ]; then
    CMD="/bin/bash"
fi 

docker_run "$DOCKER_ARGS" ${DOCKER_REGISTRY}/${DOCKER_IMAGE}:$DOCKER_TAG "$CMD"
