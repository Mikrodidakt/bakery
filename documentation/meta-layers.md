# Introduction

The builds directory and the artifacts directory will be created as part of the build while the other directories are setup as part of the project setup. How the content of the directories are setup is not defined by Bakery normally there are two ways

* git submodules
* repo android tool.

The layers directory is where the meta data used to describe the OE/Yocto project is placed. The layers directory is required while the reset is up to the project. The important is that Bakery is aware of the directories and that is where the workspace config comes in which is covered more in detail in [Workspace Config](workspace-config.md)
