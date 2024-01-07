#!/bin/bash

groupadd --gid $DOCKER_GID $DOCKER_USER
useradd -u $DOCKER_UID -g $DOCKER_USER -s /bin/bash $DOCKER_USER

if [ "$*" = "" ]; then
    su $DOCKER_USER
else
    su $DOCKER_USER -c "$*"
fi
