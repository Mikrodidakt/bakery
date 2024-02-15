# Introduction

By default bakery will use docker and it will use the bakery-workspace image. Currently bakery has no requirement except that docker is available on the system and that the user calling bakery is part of the docker group. Bakery should never run as root so if you are experiancing an permission issue when running bakery and docker then the most likely the user is not part of the docker group.

# Setup Docker

There are multiple ways to install docker on your system and bakery has currently no preference. For reference on how to setup docker please see https://github.com/Mikrodidakt/bakery/blob/main/scripts/setup-docker.sh. This script currently supports setting up Docker on ubuntu and debian it is bassed on https://docs.docker.com/engine/install/.

## Docker Group

It is a requirement that Docker is installed and setup correctly where each user belongs to the docker group to prevent running docker as root. For more information please refer to https://docs.docker.com/engine/install/linux-postinstall/. After Docker has been setup check that user belongs to the group by running

    ```bash
    groups
    ```

# Docker Pull

Currently bakery will not automatically pull down any Docker image this is in the backlog to change but currently this is a manual step. This is done by running

    ```bash
    BVERSION=$(bakery --version)
    BVERSION=${BAKERY_VERSION##* }
    docker pull strixos/bakery-workspace:${BVERSION}
    ```

# Bakery Workspace

By default the Docker image bakery-workspace is used by bakery and is uploaded to Docker hub. For some reason a Docker hub account is sometimes required to pull down even a public image. It is not clear why but if bakery-workspace cannot be pulled then your local user might be required to log into Docker hub by running

    ```bash
    docker login 
    ```

