#!/bin/bash

# Script to measure compile times for different tuple size features
# Usage: ./measure_compile_times.sh

set -e

echo "Measuring compile times for different tuple sizes..."
echo "=================================================="

# Array of features to test (empty string means default/size-8)
FEATURES=("" "size-16" "size-32" "size-48" "size-64" "size-96" "size-128")

# Test each feature
for feature in "${FEATURES[@]}"; do
    if [ -z "$feature" ]; then
        echo "Testing default (size-8):"
        time (cargo clean && cargo build --quiet)
    else
        echo "Testing $feature:"
        time (cargo clean && cargo build --quiet --features "$feature")
    fi
    echo ""
done

echo "All measurements completed!"
echo ""
echo "Update the README.md table with the new compile times."
echo "The format should be:"
echo "| Max Tuple Size | Compile Time |"
echo "|----------------|--------------|"
echo "| 8 (default)    | ~Xs          |"
echo "| 16             | ~Xs          |"
echo "| etc...         | ~Xs          |"