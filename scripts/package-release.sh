#!/bin/bash

# QM Rust Agent Release Packager
# Creates distributable archives for different platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="qmc-rust-agent"
VERSION=${1:-"$(git describe --tags --abbrev=0 2>/dev/null || echo 'v0.1.0')"}
RELEASE_DIR="releases"
BUILD_DIR="build"

print_header() {
    echo -e "${BLUE}"
    echo "======================================="
    echo "   QM Rust Agent Release Packager"
    echo "======================================="
    echo "Version: $VERSION"
    echo -e "${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ Error: $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

check_prerequisites() {
    print_info "Checking prerequisites..."

    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found"
        exit 1
    fi

    if ! command -v git &> /dev/null; then
        print_error "Git not found"
        exit 1
    fi

    if ! command -v tar &> /dev/null; then
        print_error "tar not found"
        exit 1
    fi

    print_success "All prerequisites found"
}

clean_build() {
    print_info "Cleaning previous builds..."
    rm -rf "$BUILD_DIR" "$RELEASE_DIR"
    mkdir -p "$BUILD_DIR" "$RELEASE_DIR"
    print_success "Build directories cleaned"
}

build_project() {
    print_info "Building project..."

    # Clean and build release
    cargo clean
    cargo build --release

    if [ ! -f "target/release/qm-agent" ] && [ ! -f "target/release/qm-agent.exe" ]; then
        print_error "Build failed - no binary found"
        exit 1
    fi

    print_success "Project built successfully"
}

create_source_archive() {
    print_info "Creating source archive..."

    local archive_name="${PROJECT_NAME}-${VERSION}-source"
    local temp_dir="$BUILD_DIR/$archive_name"

    # Create temporary directory
    mkdir -p "$temp_dir"

    # Copy source files (exclude build artifacts)
    rsync -av \
        --exclude='target/' \
        --exclude='.git/' \
        --exclude='releases/' \
        --exclude='build/' \
        --exclude='*.log' \
        --exclude='.DS_Store' \
        ./ "$temp_dir/"

    # Create archive
    cd "$BUILD_DIR"
    tar -czf "../$RELEASE_DIR/${archive_name}.tar.gz" "$archive_name"
    zip -r "../$RELEASE_DIR/${archive_name}.zip" "$archive_name" >/dev/null
    cd ..

    print_success "Source archive created: ${archive_name}.{tar.gz,zip}"
}

create_binary_archive() {
    local platform="$1"
    local binary_name="$2"
    local archive_name="${PROJECT_NAME}-${VERSION}-${platform}"

    print_info "Creating binary archive for $platform..."

    local temp_dir="$BUILD_DIR/$archive_name"
    mkdir -p "$temp_dir"

    # Copy binary
    cp "target/release/$binary_name" "$temp_dir/"

    # Copy essential files
    cp README.md "$temp_dir/"
    cp INSTALL.md "$temp_dir/"
    cp CLAUDE.md "$temp_dir/"
    cp LICENSE* "$temp_dir/" 2>/dev/null || echo "No LICENSE file found"

    # Copy installers
    cp install.sh "$temp_dir/" 2>/dev/null || true
    cp install.bat "$temp_dir/" 2>/dev/null || true
    chmod +x "$temp_dir/install.sh" 2>/dev/null || true

    # Copy agent configuration
    cp -r .claude "$temp_dir/" 2>/dev/null || true

    # Create quick start guide
    cat > "$temp_dir/QUICK_START.md" << EOF
# Quick Start Guide

## Installation
1. Extract this archive
2. Run the installer:
   - Unix/Linux/macOS: \`./install.sh\`
   - Windows: \`install.bat\`

## Usage
\`\`\`bash
./qm-agent minimize -i "f(A,B,C) = Σ(1,3,7)"
\`\`\`

For detailed instructions, see INSTALL.md and README.md
EOF

    # Create archive
    cd "$BUILD_DIR"
    tar -czf "../$RELEASE_DIR/${archive_name}.tar.gz" "$archive_name"
    zip -r "../$RELEASE_DIR/${archive_name}.zip" "$archive_name" >/dev/null
    cd ..

    print_success "Binary archive created: ${archive_name}.{tar.gz,zip}"
}

create_installer_only() {
    print_info "Creating installer-only package..."

    local archive_name="${PROJECT_NAME}-${VERSION}-installer"
    local temp_dir="$BUILD_DIR/$archive_name"

    mkdir -p "$temp_dir"

    # Copy source files needed for installation
    cp -r src "$temp_dir/"
    cp Cargo.toml "$temp_dir/"
    cp Cargo.lock "$temp_dir/" 2>/dev/null || true
    cp -r .claude "$temp_dir/"

    # Copy documentation and installers
    cp README.md INSTALL.md CLAUDE.md "$temp_dir/"
    cp install.sh install.bat "$temp_dir/"
    chmod +x "$temp_dir/install.sh"

    # Copy essential config files
    cp .gitignore "$temp_dir/" 2>/dev/null || true

    # Create archive
    cd "$BUILD_DIR"
    tar -czf "../$RELEASE_DIR/${archive_name}.tar.gz" "$archive_name"
    zip -r "../$RELEASE_DIR/${archive_name}.zip" "$archive_name" >/dev/null
    cd ..

    print_success "Installer package created: ${archive_name}.{tar.gz,zip}"
}

generate_checksums() {
    print_info "Generating checksums..."

    cd "$RELEASE_DIR"

    # Generate SHA256 checksums
    if command -v sha256sum &> /dev/null; then
        sha256sum *.tar.gz *.zip > "${PROJECT_NAME}-${VERSION}-checksums.txt"
    elif command -v shasum &> /dev/null; then
        shasum -a 256 *.tar.gz *.zip > "${PROJECT_NAME}-${VERSION}-checksums.txt"
    else
        print_error "No checksum utility found (sha256sum or shasum)"
    fi

    cd ..
    print_success "Checksums generated"
}

create_release_notes() {
    print_info "Creating release notes..."

    cat > "$RELEASE_DIR/RELEASE_NOTES.md" << EOF
# QM Rust Agent $VERSION

## Release Archives

### For End Users (Recommended)
- **\`${PROJECT_NAME}-${VERSION}-installer.tar.gz\`** - Source + installer (Unix/Linux/macOS)
- **\`${PROJECT_NAME}-${VERSION}-installer.zip\`** - Source + installer (Windows)

Download, extract, and run the installer:
\`\`\`bash
# Unix/Linux/macOS
./install.sh --global

# Windows
install.bat --global
\`\`\`

### Pre-built Binaries
- **\`${PROJECT_NAME}-${VERSION}-linux-x86_64.tar.gz\`** - Linux binary + docs
- **\`${PROJECT_NAME}-${VERSION}-windows-x86_64.zip\`** - Windows binary + docs
- **\`${PROJECT_NAME}-${VERSION}-macos-x86_64.tar.gz\`** - macOS binary + docs

### For Developers
- **\`${PROJECT_NAME}-${VERSION}-source.tar.gz\`** - Complete source code
- **\`${PROJECT_NAME}-${VERSION}-source.zip\`** - Complete source code

## Features

- Boolean function minimization using Quine-McCluskey algorithm
- Multiple input formats (function notation, JSON, truth tables)
- Claude Code integration as subagent
- Interactive mode and step-by-step solutions
- Cross-platform support (Windows, macOS, Linux)

## Installation

See [INSTALL.md](INSTALL.md) for detailed installation instructions.

## Verification

Checksums available in \`${PROJECT_NAME}-${VERSION}-checksums.txt\`

EOF

    print_success "Release notes created"
}

show_summary() {
    echo ""
    echo -e "${GREEN}🎉 Release packaging completed!${NC}"
    echo ""
    echo -e "${BLUE}Created archives:${NC}"
    ls -la "$RELEASE_DIR/" | grep -E '\.(tar\.gz|zip)$' | while read line; do
        echo "  $(echo $line | awk '{print $9, $5}' | sed 's/ / - /')"
    done

    echo ""
    echo -e "${BLUE}Release directory:${NC} $RELEASE_DIR/"
    echo -e "${BLUE}Total size:${NC} $(du -sh $RELEASE_DIR/ | cut -f1)"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Test the installer packages"
    echo "2. Upload to GitHub Releases"
    echo "3. Update documentation with download links"
}

main() {
    print_header

    check_prerequisites
    clean_build
    build_project

    # Create different package types
    create_source_archive
    create_installer_only

    # Create binary archives (these would need cross-compilation setup)
    # For now, just create for current platform
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        create_binary_archive "linux-x86_64" "qm-agent"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        create_binary_archive "macos-x86_64" "qm-agent"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        create_binary_archive "windows-x86_64" "qm-agent.exe"
    fi

    generate_checksums
    create_release_notes
    show_summary
}

# Run main function
main "$@"