# Introduction

Bakery consists of a number of sub-commands. Each sub-command has it's own flags for more information on what sub-command bakery supports run

```bash
user@node:/dir$ bakery help
```

For information on each sub-command and what flags are supported run

```bash
user@node:/dir$ bakery <sub-command> -h
```
# Shell

The shell subcommand will start a docker shell and setup the environment for the specified build config.

```bash
user@node:/dir$ bakery shell -c <config>
```

When starting a Bakery shell the config will be selected and used inside the shell. The terminal will present the following information

```bash
<user>@bakery-v<version>[<config>]:~/$
```

Each subcommand will be available as an alias with the build config predefined. Simply type the sub-command in the shell from any location
no need to specify bakery or the build config since it is already preset in the bakery workspace

```bash
help
build
list
deploy
upload
```

The idea with the bakery workspace shell is to have an easy environment with direct access to all the tools. If not running a sub-command in a shell most sub-commands will expect the user to specify what build config to use and it must be executed from the workspace directory containing the
build configs.


# Build

The build sub-command is for starting a build.

```bash
user@node:/dir$ bakery build -c <config>
```

The build config can consist of multiple tasks if no task is specified all that are enabled will be executed. To specify a specific task run

```bash
user@node:/dir$ bakery build -c <config> -t <task>
```

To get a list of what task a build config supports check the build config or run the [List](#List).

# Clean

The clean sub-command is for clean it will currently only remove the build directory created by the build command.

```bash
user@node:/dir$ bakery clean -c <config>
```

# List

The list sub-command will list either all the available build configs in a workspace if non is specified or a list of what tasks a build config supports if a build config is specified

```bash
user@node:/dir$ bakery list -c <config>
```

## Context

The list sub-command can also list all the context variables for a specific build config by running

```bash
user@node:/dir$ bakery list -c <config> --ctx
```

This will take the build config and list all the builtin context variables and any one defined in the build config. Can be usefull when setting up the initial workspace or debugging an issue.


# Deploy

The deploy sub-command is a special task with it's own definition in the build config. It is more or less just a proxy for calling a custom deploy script to deploy a build on the target.

```bash
user@node:/dir$ bakery deploy -c <config>
```

For details on how to configure this please see [Deploy](build-config.md#Deploy).

# Upload

The upload sub-command is a special task with it's own definition in the build config. It is more or less just a proxy for calling a custom upload script to upload to an artifact server.

```bash
user@node:/dir$ bakery upload -c <config>
```

For details on how to configure this please see [Upload](build-config.md#Upload)

# Setup

The setup sub-command is a special task with it's own definition in the build config. It is more or less just a proxy for calling a custom setup script to setup the workspace.

```bash
user@node:/dir$ bakery setup -c <config>
```

Currently the setup command is not running inside of docker so any dependency is required to be installed on the host. For details on how to configure this please see [Setup](build-config.md#Setup).

