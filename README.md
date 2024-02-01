# Bakery: Build Engine for the Yocto/OE Projects

## Introduction

Bakery is a versatile command-line tool designed to simplify and streamline the process of building Yocto Project or OpenEmbedded projects. It serves as a wrapper around
the powerful BitBake tool, leveraging Docker to ensure consistent environments across local and CI (Continuous Integration) builds.

### Why Bakery?

Building complex Yocto Project or OpenEmbedded projects can often involve intricate scripts and directory structures, making maintenance challenging. Bakery addresses this
challenge by introducing a standardized approach while preserving flexibility. It aims to enhance the development experience by consolidating essential project configuration
into JSON files, promoting a cleaner and more maintainable workflow.

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

## Installation

Bakery is written in Rust, and to install it, follow these steps:

1. Clone the Bakery repository:

    ```bash
    git clone git@github.com:Mikrodidakt/bakery.git
    cd bakery
    ```

2. Run the setup script to install Rust devtools:

    ```bash
    ./scripts/setup-rust.sh
    ```

3. Reinitialize your shell to update the environment:

    ```bash
    source ~/.bashrc   # or source ~/.zshrc, depending on your shell
    ```

4. Install Bakery using Cargo:

    ```bash
    cargo install --path .
    ```

   By default, Bakery will be installed under `~/.cargo/bin`. Ensure this path is in your `PATH` variable:

    ```bash
    export PATH=${HOME}/.cargo/bin
    ```

   If you want to install Bakery to a different system directory, change the `CARGO_HOME` variable before reinstalling:

    ```bash
    CARGO_HOME=/usr/local/cargo cargo install --path .
    ```

## Setting Up Build and Workspace Configs

To use Bakery, you need to set up the build config and workspace config files. These files describe the build configuration and the tree structure of the Bakery workspace, respectively.

### Workspace Config File

If the default workspace config is acceptable, the most basic workspace config is:

```json
{
  "version": "5"
}
```
This simple configuration will use the default directory structure and the default Bakery
Docker image (strixos/bakery-workspace:latest from Docker Hub).

### Buil Config File

TBA

## Usage

TBA

