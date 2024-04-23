# Introduction

By default, Bakery utilizes Docker and relies on the bakery-workspace image. Presently, Bakery has minimal requirements, only necessitating the availability of Docker on the system and that the user invoking Bakery is a member of the docker group. It's important to note that Bakery should never run as root. If you encounter permission issues when running Bakery with Docker, it's likely that the user is not part of the docker group.

# Setup Docker

There are multiple ways to install docker on your system and bakery has currently no preference. For reference on how to setup docker please see [setup-docker.sh](https://github.com/Mikrodidakt/bakery/blob/main/scripts/setup-docker.sh). This script currently supports setting up Docker on ubuntu and debian it is based on [Install Docker](https://docs.docker.com/engine/install/).

## Docker Group

Installing and correctly configuring Docker is a prerequisite for Bakery. It's essential that each user is a member of the docker group to avoid running Docker as root. For detailed instructions on [post-installation](https://docs.docker.com/engine/install/linux-postinstall/) steps on Linux, please refer to Post Install on Linux. Once Docker is set up, ensure that the user belongs to the docker group by running:


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

The default Docker image utilized by Bakery is bakery-workspace, which is uploaded to Docker Hub. Occasionally, even for public images, a Docker Hub account might be necessary to pull down the image. The reason for this requirement is not always clear. If you encounter issues pulling bakery-workspace, it might be necessary for your local user to log into Docker Hub by executing the following command:

```bash
user@node:/dir$ docker login
```

## Build Image

Additionally, you can build the bakery workspace locally. Before initiating a build, ensure that you are on the tag corresponding to the bakery version installed on your system.


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

The Yocto Project has established a Docker project named Crops. Although Bakery does not currently utilize it, integrating Crops containers is a future objective. These images are structured differently, and while there hasn't been an opportunity to explore their integration with Bakery yet, it is a desirable goal. Utilizing Crops containers could enhance Bakery's functionality and efficiency.

* https://hub.docker.com/r/crops/poky
* https://github.com/crops/yocto-dockerfiles
* https://github.com/crops/poky-container

