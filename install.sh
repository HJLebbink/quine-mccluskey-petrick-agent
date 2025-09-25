#!/bin/bash

# QM Rust Agent Installer
# Installs the Quine-McCluskey Boolean minimization agent for Claude Code

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
AGENT_NAME="qm-agent"
REPO_URL="https://github.com/your-username/qmc-rust-agent"

print_header() {
    echo -e "${BLUE}"
    echo "=================================="
    echo "   QM Rust Agent Installer"
    echo "=================================="
    echo -e "${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ Error: $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ Warning: $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

check_dependencies() {
    print_info "Checking dependencies..."

    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    print_success "Rust/Cargo found"

    if ! command -v git &> /dev/null; then
        print_error "Git is not installed. Please install Git first."
        exit 1
    fi
    print_success "Git found"
}

install_agent() {
    local install_type="$1"
    local target_dir=""

    case "$install_type" in
        "global")
            target_dir="$HOME/.claude/agents"
            print_info "Installing QM Agent globally for all projects..."
            ;;
        "local")
            target_dir="./.claude/agents"
            print_info "Installing QM Agent for current project only..."
            ;;
        *)
            print_error "Invalid install type. Use 'global' or 'local'"
            exit 1
            ;;
    esac

    # Create target directory
    mkdir -p "$target_dir"
    print_success "Created directory: $target_dir"

    # Copy agent configuration
    if [ -f ".claude/agents/qm-agent.md" ]; then
        cp ".claude/agents/qm-agent.md" "$target_dir/"
        print_success "Installed QM Agent configuration to $target_dir"
    else
        print_error "QM Agent configuration not found. Are you in the correct directory?"
        exit 1
    fi
}

build_binary() {
    print_info "Building QM Agent binary..."

    if cargo build --release; then
        print_success "QM Agent binary built successfully"
    else
        print_error "Failed to build QM Agent binary"
        exit 1
    fi
}

install_binary() {
    local install_type="$1"

    case "$install_type" in
        "global")
            print_info "Installing binary globally..."
            if [ -d "$HOME/.cargo/bin" ]; then
                cp "target/release/qm-agent" "$HOME/.cargo/bin/" 2>/dev/null || {
                    print_warning "Could not install to ~/.cargo/bin, binary available at: target/release/qm-agent"
                }
            else
                print_warning "~/.cargo/bin not found, binary available at: target/release/qm-agent"
            fi
            ;;
        "local")
            print_info "Binary available at: target/release/qm-agent"
            ;;
    esac
}

show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -g, --global    Install globally for all Claude Code projects"
    echo "  -l, --local     Install for current project only (default)"
    echo "  -h, --help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0              # Install locally"
    echo "  $0 --global     # Install globally"
}

main() {
    local install_type="local"

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -g|--global)
                install_type="global"
                shift
                ;;
            -l|--local)
                install_type="local"
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    print_header

    check_dependencies
    build_binary
    install_agent "$install_type"
    install_binary "$install_type"

    echo ""
    print_success "QM Rust Agent installed successfully!"
    echo ""
    print_info "Usage examples:"
    echo "  cargo run -- minimize -i \"f(A,B,C) = Σ(1,3,7)\""
    echo "  cargo run -- interactive"
    echo "  cargo run -- examples"
    echo ""
    print_info "Claude Code will automatically use this agent for Boolean minimization tasks."
    echo ""

    if [ "$install_type" = "global" ]; then
        print_info "Agent installed globally - available in all Claude Code projects"
    else
        print_info "Agent installed locally - available in current project only"
    fi
}

# Run main function with all arguments
main "$@"