[![Build & Test](https://github.com/Mikrodidakt/bakery/actions/workflows/buildntest.yml/badge.svg)](https://github.com/Mikrodidakt/bakery/actions/workflows/buildntest.yml) [![Release](https://github.com/Mikrodidakt/bakery/actions/workflows/release.yml/badge.svg)](https://github.com/Mikrodidakt/bakery/actions/workflows/release.yml)

# Bakery: Build Engine for the Yocto/OE Projects

## Introduction

Yaab is a versatile command-line tool designed to simplify and streamline the process of building Android projects. It serves as a wrapper around the powerful bitbake tool, leveraging Docker to ensure consistent environments across local and CI (Continuous Integration) builds.

### Why Yaab?

Building complex Android projects can often involve intricate scripts and directory structures, making maintenance challenging. Yaab addresses this challenge by introducing a standardized approach while preserving flexibility. It aims to enhance the development experience by consolidating essential project configuration into JSON files, promoting a cleaner and more maintainable workflow.

## Installation

### Debian Package

Download one of the deb-packages releases.

```bash
user@node:/dir$ YAAB_VERSION=x.y.z
user@node:/dir$ wget https://github.com/Mikrodidakt/bakery/releases/download/v${YAAB_VERSION}/yaab-v${YAAB_VERSION}.deb
user@node:/dir$ sudo dpkg -i yaab-v${YAAB_VERSION}.deb
```

Because yaab is written in Rust yaab is a single binary depending only on libc. It will be installed under /usr/bin/yaab.

## Build Source Code

Please see [build source code](documentation/build-yaab.md) for information on how to build yaab.

## Docker

By default, yaab utilizes Docker. Refer to the [Docker setup guide](documentation/docker.md) for detailed instructions on setting up Docker on your host system. Additionally, if you wish to run Yaab without Docker, please consult the guide on [disabling Docker](documentation/workspace-config.md#disabled) for detailed instructions. To test out that docker works run


```bash
user@node:/dir$ YAAB_VERSION=x.y.z
user@node:/dir$ docker run -it ghcr.io/mikrodidakt/bakery/bakery-workspace:${YAAB_VERSION} /bin/bash
```

## Usage

```bash
Build engine for the Yocto/OE

Usage: yaab <COMMAND>

Commands:
  sync    Sync workspace e.g sync/update git submodules
  upload  Upload artifacts to artifactory server
  deploy  Deploy artifact to target
  list    List all builds or the tasks available for one build
  setup   Setup workspace e.g initializing git submodules
  clean   Clean one or all the tasks defined in a build config
  build   Execute a build either a full build or a task of one of the builds
  shell   Initiate a shell within Docker or execute any command within the BitBake environment
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

For detailed instructions on using each sub-command, please refer to the [sub-commands](documentation/sub-commands.md) documentation. Each sub-command is configured by defining a build configuration file that describes how to build a product. It is recommended to begin by starting a shell, as this not only sets up the environment for using Bakery but also enables the use of any tools available through OE/Yocto.


```bash
user@node:/dir$ yaab shell -c <config>
```

When starting a Yaab shell the config will be selected and used inside the shell. The terminal will present the following information

```bash
<user>@yaab-v<version>[<config>]:~/$
```

Each subcommand will be available as an alias with the build config predefined. Simply type the subcommand from any location to run it. To get a list of the current subcommands run


```bash
<user>@yaab-v<version>[<config>]:~/$ help
```

To start a build run

```bash
<user>@yaab-v<version>[<config>]:~/$ build
```

For more information on how to use the shell and any sub-command please refere to [shell](documentation/sub-commands.md#shell).

## Setup A Project

To setup a project from scratch mainly four things are required

* yaab - the yaab tool. Please see [Installation](#Installation)
* workspace - the workspace config defining the workspace structure. Please see [Setup Workspace](documentation/workspace-config.md).
* build config - the build config defining how to build a product. Please see [Build Config](documentation/build-config.md).

## Key Features

- **Docker Integration:** Yaab seamlessly integrates with Docker to create reproducible build environments, ensuring consistency across different development setups.

- **JSON Configuration:** Project-specific configuration is defined in JSON files – the build config and the workspace config. The build config encapsulates all the necessary local.conf and bblayers.conf settings, simplifying the build process.

- **Task-Based Workflow:** Yaab organizes tasks within a build, allowing users to define various operations, from building image recipes to signing firmware or packaging images for redistribution. Tasks can be BitBake tasks or custom scripts, providing flexibility in project workflows.

- **Developer-Centric CI Alignment:** Yaab promotes a development environment where CI builds are defined using tasks in the build config. This ensures that CI processes are transparent and understandable, allowing developers to effortlessly reproduce the CI build locally.

    - **Consistency Across Environments:** With Yaab, the CI build process is aligned with local builds, promoting consistency. Developers can replicate CI builds effortlessly, reducing the chances of discrepancies between development and CI environments.

    - **Debugging Made Easy:** Developers can quickly identify and resolve issues by reproducing the exact CI build locally. This tight integration between CI and local development simplifies debugging, leading to faster issue resolution.

    - **Improved Developer Experience:** By aligning CI with local builds, Yaab enhances the overall developer experience. Developers gain confidence in the reliability of their builds and can iterate more efficiently on their projects.

### What Yaab Does Not Do

While Yaab simplifies many aspects of Android development, it's essential to understand its limitations:

- **Tool Replacement:** Yaab does not replace Soong or any other tools available for Android projects. It complements existing tools by wrapping some complexity to make it more straightforward for users with varying levels of knowledge.

Yaab strives to maintain the flexibility for developers who seek complete control of their projects, ensuring compatibility with all tools and workflows available in a Android project.

