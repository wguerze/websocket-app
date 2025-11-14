#!/bin/bash
# Script to format all Rust code

echo "Formatting Rust code..."
cargo fmt --all

echo ""
echo "Checking if formatting is correct..."
cargo fmt --all -- --check

if [ $? -eq 0 ]; then
    echo "✓ All code is properly formatted!"
else
    echo "✗ Code formatting issues found. Run 'cargo fmt --all' to fix."
    exit 1
fi
