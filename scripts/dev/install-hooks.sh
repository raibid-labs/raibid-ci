#!/bin/bash
# Install git hooks for raibid-ci development
# This script copies pre-commit hooks to the .git/hooks directory

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the project root
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HOOKS_SOURCE_DIR="$PROJECT_ROOT/scripts/dev/hooks"
HOOKS_TARGET_DIR="$PROJECT_ROOT/.git/hooks"

echo -e "${BLUE}Installing git hooks for raibid-ci...${NC}"
echo ""

# Check if we're in a git repository
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    echo -e "${RED}✗ Error: Not in a git repository${NC}"
    echo "This script must be run from within the raibid-ci git repository"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_TARGET_DIR"

# Define the pre-commit hook content
PRE_COMMIT_HOOK='#!/bin/bash
# Pre-commit hook for raibid-ci
# Enforces code quality checks before commits
#
# To bypass these checks, use: git commit --no-verify

set -e  # Exit on first error (unless bypassed below)

# Colors for output
RED='"'"'\033[0;31m'"'"'
GREEN='"'"'\033[0;32m'"'"'
YELLOW='"'"'\033[1;33m'"'"'
BLUE='"'"'\033[0;34m'"'"'
NC='"'"'\033[0m'"'"' # No Color

# Track if any check fails
FAILED=0

# Get the project root (two levels up from .git/hooks)
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}Running pre-commit checks...${NC}"
echo ""

# Function to print step header
print_step() {
    echo -e "${BLUE}[$1]${NC} $2"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

# Function to print error
print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Check if we'"'"'re in a merge or rebase
if [ -f "$PROJECT_ROOT/.git/MERGE_HEAD" ] || [ -f "$PROJECT_ROOT/.git/REBASE_HEAD" ]; then
    print_warning "Merge or rebase in progress, skipping pre-commit checks"
    exit 0
fi

# 1. Check code formatting
print_step "1/5" "Checking code formatting..."
if cargo fmt --all -- --check > /dev/null 2>&1; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting check failed"
    echo ""
    echo "Run the following command to fix formatting:"
    echo "  cargo fmt --all"
    echo ""
    echo "Or use: just fmt"
    FAILED=1
fi
echo ""

# 2. Run clippy
print_step "2/5" "Running clippy linter..."
if cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | grep -q "^error\|^warning"; then
    print_error "Clippy found issues"
    echo ""
    echo "Run the following command to see details:"
    echo "  cargo clippy --workspace -- -D warnings"
    echo ""
    echo "Or use: just lint"
    FAILED=1
else
    print_success "Clippy checks passed"
fi
echo ""

# 3. Run unit tests (with timeout)
print_step "3/5" "Running unit tests..."
if timeout 120s cargo test --workspace --lib --quiet 2>&1; then
    print_success "Unit tests passed"
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        print_error "Unit tests timed out (120s limit)"
    else
        print_error "Unit tests failed"
    fi
    echo ""
    echo "Run the following command to see details:"
    echo "  cargo test --workspace"
    echo ""
    echo "Or use: just test"
    FAILED=1
fi
echo ""

# 4. Validate YAML/JSON in infra directory
print_step "4/5" "Validating infrastructure files..."
YAML_ERRORS=0

# Find all YAML files in infra directory
if [ -d "$PROJECT_ROOT/infra" ]; then
    while IFS= read -r -d '"'"''"'"' file; do
        # Skip vendored charts, generated files, and Taskfiles (which use Go templates)
        if [[ "$file" == *"/vendor/"* ]] || [[ "$file" == *"/charts/"* ]] || [[ "$file" == *"Taskfile.yml"* ]]; then
            continue
        fi

        # Try to parse YAML
        if ! python3 -c "import yaml; yaml.safe_load(open('"'"'$file'"'"'))" > /dev/null 2>&1; then
            print_error "Invalid YAML: ${file#$PROJECT_ROOT/}"
            YAML_ERRORS=$((YAML_ERRORS + 1))
        fi
    done < <(find "$PROJECT_ROOT/infra" -name "*.yaml" -o -name "*.yml" -print0 2>/dev/null)
fi

# Find all JSON files in infra directory
if [ -d "$PROJECT_ROOT/infra" ]; then
    while IFS= read -r -d '"'"''"'"' file; do
        # Skip vendored files
        if [[ "$file" == *"/vendor/"* ]] || [[ "$file" == *"/node_modules/"* ]]; then
            continue
        fi

        # Try to parse JSON
        if ! python3 -c "import json; json.load(open('"'"'$file'"'"'))" > /dev/null 2>&1; then
            print_error "Invalid JSON: ${file#$PROJECT_ROOT/}"
            YAML_ERRORS=$((YAML_ERRORS + 1))
        fi
    done < <(find "$PROJECT_ROOT/infra" -name "*.json" -print0 2>/dev/null)
fi

if [ $YAML_ERRORS -eq 0 ]; then
    print_success "Infrastructure files are valid"
else
    print_error "Found $YAML_ERRORS invalid infrastructure file(s)"
    FAILED=1
fi
echo ""

# 5. Validate Nushell scripts (if nushell is installed)
print_step "5/5" "Validating Nushell scripts..."
if command -v nu > /dev/null 2>&1; then
    NU_ERRORS=0

    # Run the validate-setup script if it exists
    if [ -f "$PROJECT_ROOT/scripts/nu/validate-setup.nu" ]; then
        if nu "$PROJECT_ROOT/scripts/nu/validate-setup.nu" > /dev/null 2>&1; then
            print_success "Nushell scripts are valid"
        else
            print_error "Nushell validation failed"
            echo ""
            echo "Run the following command to see details:"
            echo "  nu scripts/nu/validate-setup.nu"
            NU_ERRORS=1
            FAILED=1
        fi
    else
        print_success "No Nushell validation script found, skipping"
    fi
else
    print_warning "Nushell not installed, skipping validation"
fi
echo ""

# Summary
echo "================================================"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All pre-commit checks passed!${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Pre-commit checks failed${NC}"
    echo ""
    echo "Fix the issues above and try again."
    echo "To bypass these checks, use: git commit --no-verify"
    echo ""
    exit 1
fi
'

# Install pre-commit hook
echo -e "${BLUE}[1/2]${NC} Installing pre-commit hook..."
echo "$PRE_COMMIT_HOOK" > "$HOOKS_TARGET_DIR/pre-commit"
chmod +x "$HOOKS_TARGET_DIR/pre-commit"
echo -e "${GREEN}✓${NC} pre-commit hook installed"
echo ""

# Verify installation
echo -e "${BLUE}[2/2]${NC} Verifying installation..."
if [ -x "$HOOKS_TARGET_DIR/pre-commit" ]; then
    echo -e "${GREEN}✓${NC} Hooks verified and executable"
else
    echo -e "${RED}✗${NC} Hook verification failed"
    exit 1
fi
echo ""

# Show information
echo "================================================"
echo -e "${GREEN}✓ Git hooks installed successfully!${NC}"
echo ""
echo "The following hooks are now active:"
echo "  - pre-commit: Runs code quality checks before each commit"
echo ""
echo "What will be checked:"
echo "  1. Code formatting (cargo fmt)"
echo "  2. Linting (cargo clippy)"
echo "  3. Unit tests (cargo test --lib)"
echo "  4. YAML/JSON validation in /infra"
echo "  5. Nushell script validation"
echo ""
echo "To bypass checks for a specific commit, use:"
echo "  git commit --no-verify"
echo ""
echo "To uninstall hooks, delete:"
echo "  .git/hooks/pre-commit"
echo ""
