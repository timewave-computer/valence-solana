#!/bin/bash

# Valence Protocol Implementation Verification Script
# Verifies the singleton architecture implementation is complete

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Verifying Valence Protocol Implementation${NC}"
echo "========================================"

# Track verification results
issues=()

# Function to check if file exists
check_file() {
    local file=$1
    local description=$2
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} $description exists"
        return 0
    else
        echo -e "${RED}✗${NC} $description missing: $file"
        issues+=("Missing: $file")
        return 1
    fi
}

# Function to check if directory exists
check_dir() {
    local dir=$1
    local description=$2
    if [ -d "$dir" ]; then
        echo -e "${GREEN}✓${NC} $description exists"
        return 0
    else
        echo -e "${RED}✗${NC} $description missing: $dir"
        issues+=("Missing: $dir")
        return 1
    fi
}

# Function to check for pattern in files
check_pattern() {
    local pattern=$1
    local path=$2
    local description=$3
    if grep -r "$pattern" "$path" --include="*.rs" --quiet 2>/dev/null; then
        echo -e "${YELLOW}⚠${NC} Found '$pattern' in $description"
        issues+=("Pattern found: $pattern in $path")
        return 1
    else
        echo -e "${GREEN}✓${NC} No '$pattern' found in $description"
        return 0
    fi
}

echo -e "\n${BLUE}Checking Core Structure...${NC}"
check_dir "programs/core/src/processor" "Processor singleton module"
check_dir "programs/core/src/scheduler" "Scheduler singleton module"
check_dir "programs/core/src/diff" "Diff singleton module"
check_dir "programs/core/src/capabilities" "Capabilities module (with embedded eval)"

echo -e "\n${BLUE}Checking Key Files...${NC}"
check_file "programs/core/src/processor/execution_engine.rs" "Processor execution engine"
check_file "programs/core/src/scheduler/queue_manager.rs" "Scheduler queue manager"
check_file "programs/core/src/diff/diff_calculator.rs" "Diff calculator"
check_file "programs/core/src/capabilities/eval_rules.rs" "Embedded eval rules"

echo -e "\n${BLUE}Checking SDK Updates...${NC}"
check_file "programs/sdk/src/processor.rs" "Processor SDK client"
check_file "programs/sdk/src/scheduler.rs" "Scheduler SDK client"
check_file "programs/sdk/src/diff.rs" "Diff SDK client"

echo -e "\n${BLUE}Checking Test Infrastructure...${NC}"
check_file "tests/integration/processor_singleton.rs" "Processor tests"
check_file "tests/integration/scheduler_singleton.rs" "Scheduler tests"
check_file "tests/integration/diff_singleton.rs" "Diff tests"
check_file "tests/integration/end_to_end.rs" "End-to-end tests"
check_file "tests/integration/performance.rs" "Performance tests"

echo -e "\n${BLUE}Checking for Legacy Code...${NC}"
if [ -d "programs/core/src/eval" ]; then
    echo -e "${RED}✗${NC} Legacy eval directory still exists"
    issues+=("Legacy directory: programs/core/src/eval")
else
    echo -e "${GREEN}✓${NC} Legacy eval directory removed"
fi

# Check for eval references (excluding valid ones)
if grep -r "programs.*eval" programs/core/src --include="*.rs" | grep -v "eval_rules" | grep -v "standalone_eval" | grep -v "// eval" 2>/dev/null; then
    echo -e "${YELLOW}⚠${NC} Found potential legacy eval references"
    issues+=("Potential legacy eval references found")
else
    echo -e "${GREEN}✓${NC} No legacy eval references found"
fi

echo -e "\n${BLUE}Checking Configuration...${NC}"
check_file "Cargo.toml" "Workspace configuration"
check_file "Anchor.toml" "Anchor configuration"
check_file ".github/workflows/test.yml" "CI/CD configuration"

echo -e "\n${BLUE}Checking Documentation...${NC}"
if [ -f "docs/101-eval.md" ]; then
    echo -e "${RED}✗${NC} Legacy eval documentation still exists"
    issues+=("Legacy doc: docs/101-eval.md")
else
    echo -e "${GREEN}✓${NC} Legacy eval documentation removed"
fi

# Summary
echo -e "\n${BLUE}Verification Summary${NC}"
echo "==================="

if [ ${#issues[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ Implementation verification passed!${NC}"
    echo "All singleton components are in place."
    exit 0
else
    echo -e "${RED}✗ Found ${#issues[@]} issues:${NC}"
    for issue in "${issues[@]}"; do
        echo "  - $issue"
    done
    exit 1
fi