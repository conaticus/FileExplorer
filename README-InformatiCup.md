# InformatiCup 2025

Hello dear team. Nice to see you here. Below is a guide to get the Explorer running. This is
partially the same as the README of the repository, except that here we are using dev mode and will
also run the tests. If there are any problems, please contact me immediately via the email address
we used to submit our project. I will take care of the issues as quickly as possible. (There
shouldn’t be any, but we have installed all the necessary dependencies on our machines, which the
instructions also include, although difficulties can always occur.)

## Complete Guide

### Requirements

- Cargo (at least version 1.80.0) -> which includes Rust
- Node.js (at least version 20.0.0)
- Tauri CLI (at least version 2.4.0)
- npm (at least version 9.0.0)

### Cloning the project

```bash
git clone https://github.com/CodeMarco05/FileExplorer
cd FileExplorer
```

### Installing dependencies

The Tauri CLI can also be installed differently. One option is a local installation using npm.
Below, installation via cargo is shown, as this is the official and Tauri-recommended method.

```bash
npm install
cargo install tauri-cli # The version should be >2.4.0 or best is 2.4.1 with the next command
cargo install tauri-cli --force --version 2.4.1
```

It may be that your environment requires additional dependencies, as shown below. These should only
be added if problems occur. Otherwise, simply continue with the first build.

### Linux

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.0-dev build-essential curl wget file libssl-dev libgtk-3-dev
```

### macOS

```bash
xcode-select --install
brew install coreutils
```

### Windows

Visual Studio build tools.

## First build

This may take several minutes depending on system performance and internet connection. Once the
build is complete, it will be indicated and the program should start immediately.

```bash
cargo tauri dev
```

This command starts the Tauri development mode, which includes all features of the Explorer.

## Building the binary

Building the binary may take several minutes. Once the build is complete, the program can be started
via the command line. The build is placed in the folder `./target/release/` with the name
`src-tauri`. (This will change later, but is still in active development.) This binary can be
executed to start the Explorer. It can also be added to system binaries so that it can be started
via the command line.

```bash
cargo tauri build
```

# Tests

The tests can be executed with the following command. All of them should pass successfully. If not,
please contact us immediately.

```bash
# First generate the necessary test data.
# This generates test data in ./src-tauri/test-data-for-fuzzy-search
# 176,840 empty files are generated, which are then used for indexing.
# Logs are created in ./src-tauri/logs/
cargo test create_test_data --features "generate-test-data" -- --nocapture

# Then run the tests
# A selection of important tests is executed, which test the functionality of the Explorer,
# but not explicitly the performance. Logs are still created for everything,
# so error logs are also created in ./src-tauri/logs/ if any occur. It is important to note that
# some errors appear there on purpose, as they are being tested.
cargo test

# To test performance and the complete feature set, the following command can be run.
# IMPORTANT: Default apps will also be opened during this test — don’t be alarmed.
# The test may also take a while. If you dont want to wait, then <Ctr-c> to stop it.
cargo test --features "full-no-generate-test-data"

# To run a specific test, you can use the following command.
cargo test <test-target-name>
```

You are welcome to review the individual tests for transparency. These are always located in the
corresponding modules. They can be found either through the console output during testing or by
looking through all source files. It’s important to note that, for example, the state of Tauri is
generated during startup. We initialize this ourselves during the tests. The source code can be
found under `./src-tauri/src/`.


# Better Testruner
You can use more advanced test runners like nextest or cargo-watch to run the tests.
In the following you can find commands to use with nextest which offer a greater range of input and output parameters.

```bash
# Install nextest
cargo install nextest
``` 

```bash
# Run all tests with nextest
cargo nextest run

# More detailed starting view
cargo nextest run --nocapture --test-threads 1 --no-fail-fast

# Run a specific test with nextest
cargo nextest run --test <test-target-name>

# You can also test how it performs when stuff is force split to multiple threads
cargo nextest run --test-threads=4
```

# Create Performance Report
To create performance reports there are multiple tools. One of them is 