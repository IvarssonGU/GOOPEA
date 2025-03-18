#!/bin/bash

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

echo "Checking dependencies..."

# Check and install wasm-pack if necessary
if ! command_exists wasm-pack; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
else
    echo "wasm-pack is already installed."
fi

# Check and install basic-http-server if necessary
if ! command_exists basic-http-server; then
    echo "Installing basic-http-server..."
    cargo install basic-http-server
else
    echo "basic-http-server is already installed."
fi

cd editor

# Run build commands
# echo "Cleaning project..."
# cargo clean

echo "Building project..."
cargo build

echo "Building WebAssembly..."
wasm-pack build --target no-modules

echo "Starting local server..."
basic-http-server

# Return to the root directory after execution
cd ..
