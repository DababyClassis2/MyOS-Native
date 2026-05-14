#!/bin/bash
# Pre-flight audit script to catch config errors locally
echo "Running Pre-flight Audit..."

echo "1. Checking Cargo features..."
cargo check --manifest-path src-tauri/Cargo.toml --all-features
if [ $? -ne 0 ]; then
    echo "ERROR: Cargo feature audit failed."
    exit 1
fi

echo "2. Verifying configuration syntax..."
# Tauri CLI check for config validity
npx tauri config validate
if [ $? -ne 0 ]; then
    echo "ERROR: Tauri config validation failed."
    exit 1
fi

echo "Audit Passed: Ready to push."
