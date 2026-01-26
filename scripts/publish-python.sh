#!/bin/bash
#
# SyntheticData Python Package Publishing Script
#
# This script publishes the datasynth-py package to PyPI.
# It handles building, verification, and publishing automatically.
#
# By default, the script checks if the version is already published
# and skips if it exists, making it safe to run multiple times.
#
# Usage:
#   ./scripts/publish-python.sh                    # Interactive (prompts for token)
#   ./scripts/publish-python.sh --dry-run          # Test without publishing
#   ./scripts/publish-python.sh --status           # Check if version is published
#   ./scripts/publish-python.sh --token <TOKEN>    # Publish with token
#   TWINE_PASSWORD=<TOKEN> ./scripts/publish-python.sh  # Publish with env var
#
# Prerequisites:
#   pip install build twine
#
# Get your token from: https://pypi.org/manage/account/token/
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
DRY_RUN=false
STATUS_ONLY=false
TOKEN=""
PYTHON_DIR="python"
PACKAGE_NAME="datasynth-py"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --status)
            STATUS_ONLY=true
            shift
            ;;
        --token)
            TOKEN="$2"
            shift 2
            ;;
        --help|-h)
            echo "SyntheticData Python Package Publisher"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dry-run      Build and verify without publishing"
            echo "  --status       Check if version is published on PyPI"
            echo "  --token TOKEN  PyPI API token (or use TWINE_PASSWORD env var)"
            echo "  --help         Show this help message"
            echo ""
            echo "Environment Variables:"
            echo "  TWINE_PASSWORD  PyPI API token (alternative to --token)"
            echo ""
            echo "Prerequisites:"
            echo "  pip install build twine"
            echo ""
            echo "Get your token from: https://pypi.org/manage/account/token/"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

print_header() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

print_step() {
    echo -e "${GREEN}▶${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✖${NC} $1"
}

print_success() {
    echo -e "${GREEN}✔${NC} $1"
}

get_version() {
    # Extract version from pyproject.toml
    grep '^version = ' "$PYTHON_DIR/pyproject.toml" | sed 's/version = "\(.*\)"/\1/'
}

check_pypi_published() {
    local version=$1
    # Check if package version exists on PyPI
    local response=$(curl -s "https://pypi.org/pypi/$PACKAGE_NAME/$version/json" 2>/dev/null)
    if echo "$response" | grep -q '"version"'; then
        return 0
    fi
    return 1
}

check_prerequisites() {
    local missing=()

    if ! command -v python3 &> /dev/null; then
        missing+=("python3")
    fi

    if ! python3 -c "import build" 2>/dev/null; then
        missing+=("build (pip install build)")
    fi

    if ! command -v twine &> /dev/null; then
        missing+=("twine (pip install twine)")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        print_error "Missing prerequisites:"
        for item in "${missing[@]}"; do
            echo "  - $item"
        done
        echo ""
        echo "Install with: pip install build twine"
        exit 1
    fi
}

clean_build() {
    print_step "Cleaning previous build artifacts..."
    rm -rf "$PYTHON_DIR/dist" "$PYTHON_DIR/build" "$PYTHON_DIR"/*.egg-info
    rm -rf "$PYTHON_DIR/datasynth_py"/*.egg-info
    print_success "Clean complete"
}

build_package() {
    print_step "Building package..."
    cd "$PYTHON_DIR"

    if python3 -m build 2>&1 | sed 's/^/  /'; then
        print_success "Build complete"
    else
        print_error "Build failed"
        exit 1
    fi

    cd ..
}

verify_package() {
    print_step "Verifying package with twine..."

    if twine check "$PYTHON_DIR/dist"/* 2>&1 | sed 's/^/  /'; then
        print_success "Package verification passed"
    else
        print_error "Package verification failed"
        exit 1
    fi
}

run_tests() {
    print_step "Running tests..."
    cd "$PYTHON_DIR"

    if python3 -m unittest discover -s tests -v 2>&1 | sed 's/^/  /'; then
        print_success "All tests passed"
    else
        print_error "Tests failed"
        exit 1
    fi

    cd ..
}

publish_package() {
    local token=$1

    print_step "Publishing to PyPI..."

    if [ -n "$token" ]; then
        export TWINE_USERNAME="__token__"
        export TWINE_PASSWORD="$token"
    elif [ -n "$TWINE_PASSWORD" ]; then
        export TWINE_USERNAME="__token__"
    else
        print_error "No PyPI token provided!"
        echo ""
        echo "Provide token via:"
        echo "  --token <TOKEN>"
        echo "  TWINE_PASSWORD=<TOKEN> environment variable"
        echo ""
        echo "Get your token from: https://pypi.org/manage/account/token/"
        exit 1
    fi

    if twine upload "$PYTHON_DIR/dist"/* 2>&1 | sed 's/^/  /'; then
        print_success "Package published successfully"
    else
        print_error "Publishing failed"
        exit 1
    fi
}

# Main script
print_header "SyntheticData Python Package Publisher"

# Change to workspace root
cd "$(dirname "$0")/.."

VERSION=$(get_version)

# Status-only mode
if [ "$STATUS_ONLY" = true ]; then
    print_header "PyPI Publishing Status"
    echo "Package: $PACKAGE_NAME"
    echo "Version: $VERSION"
    echo ""
    echo -n "Status:  "
    if check_pypi_published "$VERSION"; then
        echo -e "${GREEN}✔ published${NC}"
        echo ""
        echo "View at: https://pypi.org/project/$PACKAGE_NAME/$VERSION/"
    else
        echo -e "${YELLOW}○ not published${NC}"
        echo ""
        echo "Run to publish:"
        echo "  $0 --token <PYPI_TOKEN>"
    fi
    exit 0
fi

echo "Configuration:"
echo "  Package:     $PACKAGE_NAME"
echo "  Version:     $VERSION"
echo "  Dry run:     $DRY_RUN"
echo "  Python dir:  $PYTHON_DIR"
echo "  Working dir: $(pwd)"

# Check prerequisites
print_header "Checking Prerequisites"
check_prerequisites
print_success "All prerequisites available"

# Check if already published
print_header "Checking PyPI Status"
echo -n "$PACKAGE_NAME@$VERSION: "
if check_pypi_published "$VERSION"; then
    echo -e "${GREEN}✔ already published${NC}"
    echo ""
    print_success "Package is already published at version $VERSION"
    echo ""
    echo "View at: https://pypi.org/project/$PACKAGE_NAME/$VERSION/"
    echo ""
    echo "To publish a new version, update the version in:"
    echo "  $PYTHON_DIR/pyproject.toml"
    exit 0
else
    echo -e "${YELLOW}○ not published${NC}"
fi

# Run tests
print_header "Running Tests"
run_tests

# Clean and build
print_header "Building Package"
clean_build
build_package

# Verify
print_header "Verifying Package"
verify_package

# List build artifacts
echo ""
print_step "Build artifacts:"
ls -la "$PYTHON_DIR/dist/" | sed 's/^/  /'

# Publish or dry-run summary
if [ "$DRY_RUN" = true ]; then
    print_header "Dry Run Complete"
    print_success "Package built and verified successfully!"
    echo ""
    echo "Build artifacts are in: $PYTHON_DIR/dist/"
    echo ""
    echo "To publish for real, run:"
    echo "  $0 --token <PYPI_TOKEN>"
    echo ""
    echo "Or with environment variable:"
    echo "  TWINE_PASSWORD=<TOKEN> $0"
else
    print_header "Publishing"

    # Confirm before publishing
    echo "This will publish $PACKAGE_NAME@$VERSION to PyPI."
    echo ""
    echo -e "${YELLOW}WARNING: Publishing cannot be undone!${NC}"
    echo ""
    read -p "Continue? (yes/no) " -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        echo "Aborted. Type 'yes' to confirm."
        exit 0
    fi

    publish_package "$TOKEN"

    print_header "Summary"
    print_success "Successfully published $PACKAGE_NAME@$VERSION to PyPI!"
    echo ""
    echo "View at: https://pypi.org/project/$PACKAGE_NAME/$VERSION/"
    echo ""
    echo "Install with:"
    echo "  pip install $PACKAGE_NAME==$VERSION"
fi
