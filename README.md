[TOC]

# Bakery: Build Engine for the Yocto/OE Projects

## Introduction

Bakery is a versatile command-line tool designed to simplify and streamline the process of building Yocto Project or OpenEmbedded projects. It serves as a wrapper around the powerful BitBake tool, leveraging Docker to ensure consistent environments across local and CI (Continuous Integration) builds.

### Why Bakery?

Building complex Yocto Project or OpenEmbedded projects can often involve intricate scripts and directory structures, making maintenance challenging. Bakery addresses this challenge by introducing a standardized approach while preserving flexibility. It aims to enhance the development experience by consolidating essential project configuration into JSON files, promoting a cleaner and more maintainable workflow.

## Installation

### Debian Package

Download one of the deb-packages releases.

```bash
    BAKERY_VERSION=x.y.z
    wget https://github.com/Mikrodidakt/bakery/releases/download/v${BAKERY_VERSION}/bakery-${BAKERY_VERSION}.deb
    sudo dpkg -i bakery-${VERSION}.deb
```

Because bakery is written in Rust bakery is a single binary depending only on libc. It will be installed under /usr/bin.

### Build Source

If you are not running a Debian distribution then bakery can be built from source
by setting up rust. Follow these steps:

1. Clone the Bakery repository:

    ```bash
    git clone git@github.com:Mikrodidakt/bakery.git
    cd bakery
    ```

2. Run the setup script to install Rust devtools:

    ```bash
    ./scripts/setup-rust.sh
    ```

3. Setup cargo env:

    ```bash
    source $HOME/.cargo/env
    ```

4. Include path to PATH env

    ```bash
    export PATH=$HOME/.cargo/bin:$PATH
    ```

4. Install Bakery using Cargo:

    ```bash
    cargo install --path .
    ```

   If you want to install Bakery to a different system directory, change the `CARGO_HOME` variable before reinstalling:

    ```bash
    CARGO_HOME=/usr/local/cargo; cargo install --path .
    ```

## Usage

To try bakery out the easiest way is to use the template workspace under tests/template-workspace. Run

```bash
    cd tests/template-workspace
    git submodule init
```

This will pull down poky and together with the workspace and build config beaglebone black can be built using the bakery workspace shell covered by [Shell](###Shell)

### Setting Up Workspace

To use Bakery, you need to set up the build config and workspace config files. These files describe the build configuration and the tree structure of the Bakery workspace, respectively. The default tree structure of the Bakery workspace would be something like


```bash
    ├── artifacts
    ├── beaglebone.json
    ├── builds
    ├── docker
    ├── layers
    ├── scripts
    └── workspace.json

```

The builds directory and the artifacts directory will be created as part of the build while the other directories are setup as part of the project setup. How the content of the directories are setup is not defined by Bakery normally there are two way git submodules or repo android tool. The layers directory is where the meta data used to describe the OE/Yocto project is placed. The layers directory is required while the reset is up to the project. The important is that Bakery is aware of the directories and that is where the workspace config comes in which will be covered more in detail [Workspace Config File](##Workspace-Config-File)

### Docker

By default Bakery will use docker and by default it will use the bakery-workspace image. The bakery-workspace will contain a Bakery version matching the version of the image. When running a Bakery subcommand Bakery will be bootstrapped into docker and will execute the subcommand inside the docker container. It is a requirement that docker
is installed and setup correctly where each user belongs to the docker group to prevent running docker as root. For reference on how to setup docker please see https://github.com/Mikrodidakt/bakery/blob/main/scripts/setup-docker.sh which could be used as a reference.

### List

To list what builds are supported in a workspace run

```bash
    bakery list
```

This will list all available builds in the Bakery workspace including a short description.

### Shell

The prefered way of working with Bakery is to start a shell this will setup the env for using Bakery but also to make it possible to run any OE/Yocto bitbake
command.

```bash
    bakery shell -c <config>
```

When starting a Bakery shell the config will be set and a couple of aliases will be available inside the shell to prevent from having to specify the build config everytime. The terminal will present the following information

```bash
    <user>@bakery-v<version>[<config>]:~/$
```

Each subcommand will be available as an alias with the build config predefined. Simply type the subcommand to run it. To get a list of the current subcommands run

```bash
    <user>@bakery-v<version>[<config>]:~/$ help
```

To start a build run

```bash
    <user>@bakery-v<version>[<config>]:~/$ build
```

## Workspace Config File

If the default workspace config is acceptable, the most basic workspace config is:

```json
    {
        "version": "5"
    }
```

This simple configuration will use the default directory structure and the default Bakery Docker image (strixos/bakery-workspace:latest). For more details on the workspace config format please see [link]

## Build Config File

For a simple build config please see tests/template-workspace/beaglebone.json. The details for how to setup a build config can be found at [link]

## Key Features

- **Docker Integration:** Bakery seamlessly integrates with Docker to create reproducible build environments, ensuring consistency across different development setups.

- **JSON Configuration:** Project-specific configuration is defined in JSON files – the build config and the workspace config. The build config encapsulates all the necessary local.conf and bblayers.conf settings, simplifying the build process.

- **Task-Based Workflow:** Bakery organizes tasks within a build, allowing users to define various operations, from building image recipes to signing firmware or packaging images for redistribution. Tasks can be BitBake tasks or custom scripts, providing flexibility in project workflows.

- **Developer-Centric CI Alignment:** Bakery promotes a development environment where CI builds are defined using tasks in the build config. This ensures that CI processes are transparent and understandable, allowing developers to effortlessly reproduce the CI build locally.

    - **Consistency Across Environments:** With Bakery, the CI build process is aligned with local builds, promoting consistency. Developers can replicate CI builds effortlessly, reducing the chances of discrepancies between development and CI environments.

    - **Debugging Made Easy:** Developers can quickly identify and resolve issues by reproducing the exact CI build locally. This tight integration between CI and local development simplifies debugging, leading to faster issue resolution.

    - **Improved Developer Experience:** By aligning CI with local builds, Bakery enhances the overall developer experience. Developers gain confidence in the reliability of their builds and can iterate more efficiently on their projects.

### What Bakery Does Not Do

While Bakery simplifies many aspects of Yocto Project and OpenEmbedded development, it's essential to understand its limitations:

- **Meta Layer Setup:** Bakery does not handle the setup of meta layers. Developers are encouraged to use Git submodules or the Android Repo tool for managing meta layers independently.
- **Tool Replacement:** Bakery does not replace BitBake or any other tools available for Yocto Project or OpenEmbedded projects. It complements existing tools by wrapping some complexity to make it more straightforward for users with varying levels of knowledge.

Bakery strives to maintain the flexibility for developers who seek complete control of their projects, ensuring compatibility with all tools and workflows available in a Yocto Project and OpenEmbedded project.

