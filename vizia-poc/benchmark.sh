#!/bin/bash
# Compile time benchmark script

set -e

echo "======================================"
echo "Vizia PoC Compile Time Benchmark"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Clean build
echo -e "${BLUE}[1/3] Clean build...${NC}"
cargo clean
echo ""
echo "Starting clean build timer..."
time cargo build 2>&1 | grep -v "Compiling" | grep -v "Finished" || true
echo ""

# Incremental build (no changes)
echo -e "${BLUE}[2/3] Incremental build (no changes)...${NC}"
echo "Starting incremental build timer..."
time cargo build 2>&1 | grep -v "Finished" || true
echo ""

# Incremental build (touch UI file)
echo -e "${BLUE}[3/3] Incremental build (touch UI file)...${NC}"
touch src/ui/keyboard_page.rs
echo "Starting incremental build timer (UI file modified)..."
time cargo build 2>&1 | grep -v "Compiling" | grep -v "Finished" || true
echo ""

echo -e "${GREEN}Benchmark complete!${NC}"
echo ""
echo "Compare these times with the Slint baseline from the main project."
echo "See README.md for decision matrix."
