# Introduction

Bakery is built using Rust. To build the source code and use it instead of one of the prebuilt deb-files the Rust tools needs to be installed. Feel free to use any Rust version but the version used can be found in the setup-rust.sh script. Bakery has been developed on ubuntu 23.04.

# Build

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


