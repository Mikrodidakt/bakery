[TOC]

# Bakery: Build Engine for the Yocto/OE Projects

## Introduction

Bakery is a versatile command-line tool designed to simplify and streamline the process of building Yocto Project or OpenEmbedded projects. It serves as a wrapper around the powerful bitbake tool, leveraging Docker to ensure consistent environments across local and CI (Continuous Integration) builds.

### Why Bakery?

Building complex Yocto Project or OpenEmbedded projects can often involve intricate scripts and directory structures, making maintenance challenging. Bakery addresses this challenge by introducing a standardized approach while preserving flexibility. It aims to enhance the development experience by consolidating essential project configuration into JSON files, promoting a cleaner and more maintainable workflow.

## Installation

### Debian Package

Download one of the deb-packages releases.

```bash
user@node:/dir$ BAKERY_VERSION=x.y.z
user@node:/dir$ wget https://github.com/Mikrodidakt/bakery/releases/download/v${BAKERY_VERSION}/bakery-${BAKERY_VERSION}.deb
user@node:/dir$ sudo dpkg -i bakery-${VERSION}.deb
```

Because bakery is written in Rust bakery is a single binary depending only on libc. It will be installed under /usr/bin.

### Build Source Code

Please see [build source code](documentation/build-bakery.md) for information on how to build bakery.

## Docker

By default bakery will use Docker. Please see [how to setup docker](documentation/docker.md) for more information on setup docker. Please see [how to disable docker](documentation/workspace-config.md#disabled) for information on how to run bakery without docker. 

## Usage

To try bakery out the easiest way is to use the template workspace under tests/template-workspace. Run

```bash
user@node:/dir$ cd tests/template-workspace
user@node:/dir$ git submodule init
```

This will pull down poky and together with the workspace and the build config beaglebone black can be built using the bakery workspace shell covered by [Shell](#Shell)

### List

To list what builds are supported in a workspace run

```bash
user@node:/dir$ bakery list
```

This will list all available builds in the Bakery workspace including a short description.

### Shell

The prefered way of working with Bakery is to start a shell this will setup the env for using Bakery but also to make it possible to run any tools available by OE/Yocto.

```bash
user@node:/dir$ bakery shell -c <config>
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

For more information on how to use each shell sub-command please refere to [shell](documentation/sub-commands.md#shell).

## Sub-Commands

For a detailed description of what sub-commands bakery offers please run

```bash
user@node:/dir$ bakery help
```

For information on each sub-command and what flags it supports run

```bash
user@node:/dir$ bakery <sub-command> --help
```

For more information on how to use each sub-command please refere to [sub-commands](documentation/sub-commands.md).

## Setup A Project

To setup a project from scratch mainly four things are required

* bakery - the bakery tool. Please see [Installation](#Installation)
* workspace - the workspace config defining the workspace structure. Please see [Setup Workspace](documentation/workspace-config.md).
* build config - the build config defining how to build a product. Please see [Build Config](documentation/build-config.md).
* meta layers - the meta data used by bitbake when producing the artifacts needed by the product. Please see [Meta Layers](documentation/meta-layers.md).

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

