#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. $SCRIPT_DIR/lib_docker.sh
. ${SCRIPT_DIR}/../scripts/lib.sh
NP_DOCKER_ARGS="$1"
DOCKER_REGISTRY=$(get_docker_registry $WORKSPACE/settings.json)
DOCKER_IMAGE=$(get_docker_image $WORKSPACE/settings.json)
DOCKER_DIR=$(get_docker_dir $WORKSPACE/settings.json)
DOCKER_TAG=$(get_yaab_version $WORKSPACE/Cargo.toml)

help()
{
    echo ""
    echo "Docker build script"
    echo ""
    echo "$0 <args>"
    echo ""
    echo "The following args are supported:"
    echo ""
    echo "help - print this text"
    echo ""
}

case $NP_DOCKER_ARGS in
    "help")
        help
        exit 0
    ;;
esac

# If running this script inside docker we will terminate
# it is not supported currently to build a docker image
# inside docker
docker_inside

docker_build_image ${WORKSPACE}/settings.json ${DOCKER_REGISTRY} ${DOCKER_IMAGE} $DOCKER_TAG
