#!/bin/bash
# MediaOrganizer Build Script

set -e  # Exit on any error

echo "🚀 MediaOrganizer Build Script"
echo "==============================="

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTION]"
    echo "Options:"
    echo "  dev         Build for development (default)"
    echo "  prod        Build for production with optimizations"  
    echo "  test        Run tests"
    echo "  clean       Clean build artifacts"
    echo "  check       Check code without building"
    echo "  lint        Run clippy linter"
    echo "  format      Format code"
    echo "  all         Run format, lint, test, and build"
    echo "  --help      Show this help message"
}

# Parse command line arguments
BUILD_TYPE=${1:-dev}

case $BUILD_TYPE in
    dev)
        echo "🔧 Building for development..."
        cargo build --no-default-features
        ;;
    prod)
        echo "🚀 Building for production..."
        cargo build --release --no-default-features
        echo "✅ Production build completed!"
        echo "📦 Binary location: target/release/media-organizer"
        ;;
    test)
        echo "🧪 Running tests..."
        cargo test --no-default-features
        ;;
    clean)
        echo "🧹 Cleaning build artifacts..."
        cargo clean
        echo "✅ Clean completed!"
        ;;
    check)
        echo "🔍 Checking code..."
        cargo check --no-default-features
        ;;
    lint)
        echo "📋 Running clippy linter..."
        cargo clippy --no-default-features -- -D warnings
        ;;
    format)
        echo "✨ Formatting code..."
        cargo fmt
        echo "✅ Code formatted!"
        ;;
    all)
        echo "🔄 Running complete build pipeline..."
        
        echo "1️⃣ Formatting code..."
        cargo fmt
        
        echo "2️⃣ Running clippy linter..."
        cargo clippy --no-default-features -- -D warnings
        
        echo "3️⃣ Running tests..."
        cargo test --no-default-features
        
        echo "4️⃣ Building for development..."
        cargo build --no-default-features
        
        echo "✅ Complete build pipeline finished!"
        ;;
    --help)
        show_usage
        exit 0
        ;;
    *)
        echo "❌ Error: Unknown option '$BUILD_TYPE'"
        show_usage
        exit 1
        ;;
esac

echo "✅ Build completed successfully!"