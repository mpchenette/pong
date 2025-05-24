#!/bin/bash

# Ensure we use rustup's toolchain
export PATH="$HOME/.cargo/bin:$PATH"

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Add wasm32 target if using rustup
if command -v rustup &> /dev/null; then
    echo "Adding wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Build the Rust project to WebAssembly
echo "Building Rust project to WebAssembly..."
wasm-pack build --target web --out-dir pkg

if [ $? -ne 0 ]; then
    echo "Build failed. Make sure you have the wasm32-unknown-unknown target installed."
    echo "If using rustup: rustup target add wasm32-unknown-unknown"
    echo "If using system Rust: see https://rustwasm.github.io/wasm-pack/book/prerequisites/non-rustup-setups.html"
    exit 1
fi

# Start a simple HTTP server
echo "Starting development server..."
echo "Open your browser to http://localhost:8000"

# Check if Python 3 is available, otherwise use Python 2
if command -v python3 &> /dev/null; then
    python3 -m http.server 8000
elif command -v python &> /dev/null; then
    python -m SimpleHTTPServer 8000
else
    echo "Python not found. Please install Python or use another HTTP server."
    echo "You can also use: npx serve . or any other static file server"
fi
