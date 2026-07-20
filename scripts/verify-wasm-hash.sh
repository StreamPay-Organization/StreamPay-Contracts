#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration - FIXED: Correct contract name from Cargo.toml
CONTRACT_NAME="streampay_contract"  # This matches the name in Cargo.toml
WASM_DIR="./target/wasm32-unknown-unknown/release"
WASM_FILE="${WASM_DIR}/${CONTRACT_NAME}.wasm"
EXPECTED_HASH_FILE="./.deployed-hash"
NETWORK="${1:-testnet}"

# Function to find WASM file automatically
find_wasm_file() {
    if [ -f "$WASM_FILE" ]; then
        echo "$WASM_FILE"
        return 0
    fi
    
    # Try deps folder
    if [ -f "${WASM_DIR}/deps/${CONTRACT_NAME}.wasm" ]; then
        echo "${WASM_DIR}/deps/${CONTRACT_NAME}.wasm"
        return 0
    fi
    
    # Try any wasm file
    local found=$(find ./target -name "*.wasm" -type f 2>/dev/null | head -1)
    if [ -n "$found" ]; then
        echo "$found"
        return 0
    fi
    
    return 1
}

# Help function
show_help() {
    echo "Usage: ./scripts/verify-wasm-hash.sh [network] [options]"
    echo ""
    echo "Options:"
    echo "  network      Network to verify (mainnet|testnet|devnet) - default: testnet"
    echo "  --build      Build the WASM file before verification"
    echo "  --save       Save the computed hash as the expected hash"
    echo "  --help       Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./scripts/verify-wasm-hash.sh mainnet"
    echo "  ./scripts/verify-wasm-hash.sh testnet --build"
    echo "  ./scripts/verify-wasm-hash.sh devnet --save"
}

# Print colored output
print_info() { echo -e "${BLUE}ℹ${NC} $1"; }
print_success() { echo -e "${GREEN}✅${NC} $1"; }
print_warning() { echo -e "${YELLOW}⚠️${NC} $1"; }
print_error() { echo -e "${RED}❌${NC} $1"; }

# Check if WASM file exists and set path
check_wasm_file() {
    local found_path=$(find_wasm_file)
    
    if [ -n "$found_path" ] && [ -f "$found_path" ]; then
        WASM_FILE="$found_path"
        print_info "Found WASM file at: $WASM_FILE"
        return 0
    fi
    
    print_error "WASM file not found at: $WASM_FILE"
    print_info "Run the script with --build to build the WASM file"
    return 1
}

# Calculate SHA-256 hash of WASM file
calculate_hash() {
    if command -v sha256sum &> /dev/null; then
        sha256sum "$WASM_FILE" | awk '{print $1}'
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "$WASM_FILE" | awk '{print $1}'
    else
        print_error "No SHA-256 tool found (sha256sum or shasum required)"
        return 1
    fi
}

# Build the WASM file
build_wasm() {
    print_info "Building WASM file..."
    
    # Build with cargo
    cargo build --target wasm32-unknown-unknown --release
    
    # Verify the WASM was created
    local found_path=$(find_wasm_file)
    if [ -n "$found_path" ]; then
        WASM_FILE="$found_path"
        print_success "WASM built successfully at: $WASM_FILE"
        
        # Show file size
        local file_size=$(du -h "$WASM_FILE" | awk '{print $1}')
        print_info "File size: $file_size"
    else
        print_error "WASM build may have failed - file not found"
        return 1
    fi
}

# Get expected hash from various sources
get_expected_hash() {
    local hash=""
    
    # Check for environment variable
    if [ -n "$EXPECTED_WASM_HASH" ]; then
        echo "$EXPECTED_WASM_HASH"
        return 0
    fi
    
    # Check for network-specific hash file
    local hash_file="${EXPECTED_HASH_FILE}.${NETWORK}"
    if [ -f "$hash_file" ]; then
        cat "$hash_file"
        return 0
    fi
    
    # Check for default hash file
    if [ -f "$EXPECTED_HASH_FILE" ]; then
        cat "$EXPECTED_HASH_FILE"
        return 0
    fi
    
    print_error "No expected hash found. Please set EXPECTED_WASM_HASH or provide hash file."
    return 1
}

# Save hash to file
save_hash() {
    local hash="$1"
    local hash_file="${EXPECTED_HASH_FILE}.${NETWORK}"
    
    echo "$hash" > "$hash_file"
    print_success "Hash saved to $hash_file"
    
    # Also save to main file if it doesn't exist
    if [ ! -f "$EXPECTED_HASH_FILE" ]; then
        echo "$hash" > "$EXPECTED_HASH_FILE"
        print_info "Also saved to $EXPECTED_HASH_FILE"
    fi
}

# Main verification function
verify() {
    echo ""
    echo "========================================"
    echo "  WASM Hash Verification"
    echo "  Network: $NETWORK"
    echo "  Contract: $CONTRACT_NAME"
    echo "========================================"
    echo ""
    
    # Check if WASM file exists
    if ! check_wasm_file; then
        return 1
    fi
    
    # Get expected hash
    local expected_hash
    expected_hash=$(get_expected_hash)
    if [ -z "$expected_hash" ]; then
        return 1
    fi
    
    # Calculate actual hash
    local actual_hash
    actual_hash=$(calculate_hash)
    if [ -z "$actual_hash" ]; then
        return 1
    fi
    
    # Get file size
    local file_size=$(du -h "$WASM_FILE" | awk '{print $1}')
    
    echo "📄 WASM File: $WASM_FILE"
    echo "📦 File Size: $file_size"
    echo "📊 Expected Hash: $expected_hash"
    echo "📊 Actual Hash:   $actual_hash"
    echo ""
    
    # Compare hashes
    if [ "$actual_hash" = "$expected_hash" ]; then
        print_success "Verification PASSED! Hashes match."
        echo ""
        echo "The deployed contract matches the expected version."
        return 0
    else
        print_error "Verification FAILED! Hashes do not match."
        echo ""
        echo "The deployed contract does NOT match the expected version."
        echo "This may indicate:"
        echo "  - The contract was updated without updating the expected hash"
        echo "  - A different version was deployed"
        echo "  - The contract was tampered with"
        echo ""
        echo "To update the expected hash:"
        echo "  ./scripts/verify-wasm-hash.sh $NETWORK --save"
        return 1
    fi
}

# Parse arguments
BUILD=false
SAVE=false

for arg in "$@"; do
    case $arg in
        --build)
            BUILD=true
            ;;
        --save)
            SAVE=true
            ;;
        --help)
            show_help
            exit 0
            ;;
        mainnet|testnet|devnet)
            NETWORK="$arg"
            ;;
        *)
            # Unknown argument
            ;;
    esac
done

# Build if requested
if [ "$BUILD" = true ]; then
    build_wasm
fi

# Save hash if requested
if [ "$SAVE" = true ]; then
    if check_wasm_file; then
        # FIXED: Removed 'local' keyword here
        hash=$(calculate_hash)
        if [ -n "$hash" ]; then
            save_hash "$hash"
            echo ""
            echo "Hash: $hash"
        fi
    else
        print_error "Cannot save hash: WASM file not found"
        print_info "Build the WASM file first: ./scripts/verify-wasm-hash.sh --build"
        exit 1
    fi
fi

# Run verification
if verify; then
    exit 0
else
    exit 1
fi