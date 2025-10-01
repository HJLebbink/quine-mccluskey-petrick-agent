#!/bin/bash
# Script to benchmark MAX_16_BITS = true vs false
# This automates the process of running benchmarks for both configurations

set -e

CLASSIC_RS="src/qm/classic.rs"
BACKUP_FILE="${CLASSIC_RS}.backup"
RESULTS_DIR="benches/results"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== MAX_16_BITS Performance Benchmark ===${NC}"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Backup original file
echo -e "${YELLOW}Creating backup of $CLASSIC_RS${NC}"
cp "$CLASSIC_RS" "$BACKUP_FILE"

# Function to set MAX_16_BITS
set_max_16_bits() {
    local value=$1
    local label=$2
    echo -e "${YELLOW}Setting MAX_16_BITS = $value ($label mode)${NC}"
    sed -i "s/pub const MAX_16_BITS: bool = .*/pub const MAX_16_BITS: bool = $value;/" "$CLASSIC_RS"
}

# Function to run benchmarks
run_benchmark() {
    local baseline=$1
    local label=$2
    echo -e "${GREEN}Running benchmarks with $label...${NC}"
    echo "This may take 5-10 minutes..."

    # Run with --save-baseline
    cargo bench --bench max_16_bits_bench -- --save-baseline "$baseline" 2>&1 | tee "${RESULTS_DIR}/raw_${baseline}.txt"

    echo -e "${GREEN}✓ Benchmarks complete for $label${NC}"
    echo ""
}

# Step 1: Benchmark 32-bit mode
set_max_16_bits "false" "32-bit"
run_benchmark "32bit" "32-bit mode"

# Step 2: Benchmark 16-bit mode
set_max_16_bits "true" "16-bit"
run_benchmark "16bit" "16-bit mode"

# Restore original file
echo -e "${YELLOW}Restoring original $CLASSIC_RS${NC}"
mv "$BACKUP_FILE" "$CLASSIC_RS"

# Compare results using critcmp if available
echo ""
echo -e "${GREEN}=== Comparing Results ===${NC}"
if command -v critcmp &> /dev/null; then
    echo "Using critcmp for comparison:"
    critcmp 32bit 16bit | tee "${RESULTS_DIR}/comparison.txt"
else
    echo -e "${YELLOW}Note: Install critcmp for detailed comparison:${NC}"
    echo "  cargo install critcmp"
    echo ""
    echo "Then run:"
    echo "  critcmp 32bit 16bit"
fi

echo ""
echo -e "${GREEN}=== Benchmark Complete ===${NC}"
echo "Results saved in: $RESULTS_DIR/"
echo "- raw_32bit.txt: Full output for 32-bit mode"
echo "- raw_16bit.txt: Full output for 16-bit mode"
echo "- comparison.txt: Side-by-side comparison (if critcmp installed)"
echo ""
echo "Criterion HTML reports available at:"
echo "  target/criterion/report/index.html"
