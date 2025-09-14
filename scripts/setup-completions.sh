#!/bin/bash

# CompressCLI Shell Completion Setup Script
# This script helps you set up autocompletion for compresscli

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Detect shell
detect_shell() {
    # Check parent process to detect the actual shell being used
    local parent_shell
    if command -v ps >/dev/null 2>&1; then
        # Try to get parent process name
        parent_shell=$(ps -p $PPID -o comm= 2>/dev/null | tr -d ' ')
        case "$parent_shell" in
            *zsh*) echo "zsh"; return ;;
            *bash*) echo "bash"; return ;;
            *fish*) echo "fish"; return ;;
        esac
    fi
    
    # Fallback to checking $SHELL environment variable
    case "$SHELL" in
        */zsh) echo "zsh" ;;
        */bash) echo "bash" ;;
        */fish) echo "fish" ;;
        *) 
            # Last resort: check if common shell executables exist and guess
            if command -v zsh >/dev/null 2>&1 && [ -f "$HOME/.zshrc" ]; then
                echo "zsh"
            elif command -v bash >/dev/null 2>&1 && [ -f "$HOME/.bashrc" ]; then
                echo "bash"
            elif command -v fish >/dev/null 2>&1; then
                echo "fish"
            else
                echo "unknown"
            fi
            ;;
    esac
}

# Build compresscli if needed
build_compresscli() {
    print_header "Building CompressCLI"
    
    cd "$PROJECT_DIR"
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust first."
        exit 1
    fi
    
    cargo build --release
    print_success "CompressCLI built successfully"
}

# Generate completion script
generate_completion() {
    local shell="$1"
    local output_file="$2"
    
    print_header "Generating $shell completion script"
    
    cd "$PROJECT_DIR"
    cargo run --release -- completions "$shell" > "$output_file"
    print_success "Generated completion script: $output_file"
}

# Setup bash completion
setup_bash() {
    print_header "Setting up Bash completion"
    
    local completion_file="$HOME/.local/share/bash-completion/completions/compresscli"
    
    # Create directory if it doesn't exist
    mkdir -p "$(dirname "$completion_file")"
    
    # Generate completion script
    generate_completion "bash" "$completion_file"
    
    # Add to bashrc if not already present
    local bashrc="$HOME/.bashrc"
    if [ -f "$bashrc" ]; then
        if ! grep -q "bash-completion" "$bashrc"; then
            echo "" >> "$bashrc"
            echo "# Enable bash completion" >> "$bashrc"
            echo "if [ -f /usr/share/bash-completion/bash_completion ]; then" >> "$bashrc"
            echo "    . /usr/share/bash-completion/bash_completion" >> "$bashrc"
            echo "elif [ -f /etc/bash_completion ]; then" >> "$bashrc"
            echo "    . /etc/bash_completion" >> "$bashrc"
            echo "fi" >> "$bashrc"
            print_success "Added bash-completion setup to ~/.bashrc"
        fi
    fi
    
    print_success "Bash completion installed"
    print_warning "Please restart your shell or run: source ~/.bashrc"
}

# Setup zsh completion
setup_zsh() {
    print_header "Setting up Zsh completion"
    
    # Create completion directory if it doesn't exist
    local comp_dir="$HOME/.local/share/zsh/site-functions"
    mkdir -p "$comp_dir"
    
    local completion_file="$comp_dir/_compresscli"
    
    # Generate completion script
    generate_completion "zsh" "$completion_file"
    
    # Add to zshrc if not already present
    local zshrc="$HOME/.zshrc"
    if [ -f "$zshrc" ]; then
        if ! grep -q "fpath.*$comp_dir" "$zshrc"; then
            echo "" >> "$zshrc"
            echo "# Add custom completion directory" >> "$zshrc"
            echo "fpath=(\"$comp_dir\" \$fpath)" >> "$zshrc"
            echo "autoload -U compinit && compinit" >> "$zshrc"
            print_success "Added completion setup to ~/.zshrc"
        fi
    fi
    
    print_success "Zsh completion installed"
    print_warning "Please restart your shell or run: source ~/.zshrc"
}

# Setup fish completion
setup_fish() {
    print_header "Setting up Fish completion"
    
    local comp_dir="$HOME/.config/fish/completions"
    mkdir -p "$comp_dir"
    
    local completion_file="$comp_dir/compresscli.fish"
    
    # Generate completion script
    generate_completion "fish" "$completion_file"
    
    print_success "Fish completion installed"
    print_warning "Fish will automatically load the completion on next startup"
}

# Setup PowerShell completion (for Windows users with WSL)
setup_powershell() {
    print_header "Setting up PowerShell completion"
    
    local completion_file="$HOME/compresscli_completion.ps1"
    
    # Generate completion script
    generate_completion "powershell" "$completion_file"
    
    print_success "PowerShell completion generated: $completion_file"
    print_warning "To use in PowerShell, add this to your profile:"
    echo "    . $completion_file"
}

# Main function
main() {
    print_header "CompressCLI Autocompletion Setup"
    
    # Build the project first
    build_compresscli
    
    # Detect shell or use provided argument
    local shell="${1:-$(detect_shell)}"
    
    case "$shell" in
        bash)
            setup_bash
            ;;
        zsh)
            setup_zsh
            ;;
        fish)
            setup_fish
            ;;
        powershell)
            setup_powershell
            ;;
        all)
            setup_bash
            setup_zsh
            setup_fish
            setup_powershell
            ;;
        *)
            print_error "Unsupported shell: $shell"
            echo "Supported shells: bash, zsh, fish, powershell, all"
            echo "Usage: $0 [shell]"
            exit 1
            ;;
    esac
    
    print_header "Setup Complete!"
    echo "Autocompletion for $shell has been configured."
    echo ""
    echo "To test it, try typing:"
    echo "  compresscli <TAB>"
    echo "  compresscli video --<TAB>"
    echo "  compresscli image --format <TAB>"
    echo ""
    echo "For more information, see the 'Shell Autocompletion' section in README.md"
}

# Run main function
main "$@"