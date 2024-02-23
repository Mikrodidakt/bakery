# Introduction

By default bakery will use docker and it will use the bakery-workspace image. Currently bakery has no requirement except that docker is available on the system and that the user calling bakery is part of the docker group. Bakery should never run as root so if you are experiancing an permission issue when running bakery and docker then the most likely the user is not part of the docker group.

# Setup Docker

There are multiple ways to install docker on your system and bakery has currently no preference. For reference on how to setup docker please see [setup-docker.sh](https://github.com/Mikrodidakt/bakery/blob/main/scripts/setup-docker.sh). This script currently supports setting up Docker on ubuntu and debian it is bassed on [Install Docker](https://docs.docker.com/engine/install/).

## Docker Group

It is a requirement that Docker is installed and setup correctly where each user belongs to the docker group to prevent running docker as root. For more information please refer to [Post Install on Linux](https://docs.docker.com/engine/install/linux-postinstall/). After Docker has been setup check that user belongs to the group by running

```bash
user@node:/dir$ groups
user adm cdrom sudo dip plugdev docker
```

# Docker Pull

Currently bakery will not automatically pull down any Docker image this is in the backlog to change but currently this is a manual step. This is done by running

```bash
user@node:/dir$ BVERSION=$(bakery --version)
user@node:/dir$ BVERSION=${BAKERY_VERSION##* }
user@node:/dir$ docker pull strixos/bakery-workspace:${BVERSION}
```

# Bakery Workspace Image

## Docker Hub

By default the Docker image bakery-workspace is used by bakery and is uploaded to Docker hub. For some reason a Docker hub account is sometimes required to pull down even a public image. It is not clear why but if bakery-workspace cannot be pulled then your local user might be required to log into Docker hub by running

```bash
user@node:/dir$ docker login
```

## Build Image

The bakery workspace can also be built locally beofre starting a build make sure that you are on the tag matching the bakery that you have installed on your system.


```bash
user@node:/dir$ bakery --version
```

Once that is done simply run the following command from the bakery repo.


```bash
user@node:/dir$ ./docker/do_docker_build.sh
```

after it has completed you can run


```bash
user@node:/dir$ docker images
REPOSITORY                      TAG       IMAGE ID       CREATED        SIZE
strixos/bakery-workspace        0.1.36    f896c2e2b7f7   8 days ago     2.58GB
strixos/bakery-workspace        latest    f896c2e2b7f7   8 days ago     2.58GB

```

# Crops

The Yocto Project has setup a docker project called crops currently the bakery is not using it but that would be the goal. The images are setup a bit differently and I need to go over it to use if for bakery and have simply not had time yet. But it would be greate if the bakery could make use of the crops containers.

* https://hub.docker.com/r/crops/poky
* https://github.com/crops/yocto-dockerfiles
* https://github.com/crops/poky-container

