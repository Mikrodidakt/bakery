# Introduction

## Setting Up Workspace

To use bakery, you need to set up the build config and workspace config files. These files describe the build configuration and the tree structure of the Bakery workspace, respectively. The default tree structure of the Bakery workspace would be something like


```bash
    ├── artifacts
    ├── beaglebone.json
    ├── builds
    ├── docker
    ├── layers
    ├── scripts
    └── workspace.json

```

The layers directory is where the meta data used to describe the OE/Yocto project is placed for more information on this topic please refere to [Meta Layers](meta-layers.md). The layers directory is required while the reset is up to the project. The important is that Bakery is aware of the directories and that is where the workspace config comes in.

### Workspace Config File

If the default workspace config is acceptable, the most basic workspace config is:

```json
    {
        "version": "5"
    }
```

This simple configuration will use the default directory structure and the default Bakery Docker image (strixos/bakery-workspace:<version>). For more details on the workspace config format please see [link]


# Disable Docker
