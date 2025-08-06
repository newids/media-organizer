# MediaOrganizer Makefile

.PHONY: help build dev prod test clean check lint format run install

# Default target
help:
	@echo "MediaOrganizer Build System"
	@echo "=========================="
	@echo "Available targets:"
	@echo "  dev      - Build for development (default features disabled)"
	@echo "  prod     - Build for production with optimizations" 
	@echo "  test     - Run all tests"
	@echo "  check    - Check code without building"
	@echo "  lint     - Run clippy linter"
	@echo "  format   - Format code with rustfmt"
	@echo "  clean    - Clean build artifacts"
	@echo "  run      - Run the application in development mode"
	@echo "  install  - Install system dependencies"
	@echo "  all      - Run complete build pipeline"

# Development build (no optional features due to FFmpeg issues)
dev:
	@echo "🔧 Building for development..."
	cargo build --no-default-features

# Production build
prod:
	@echo "🚀 Building for production..."
	cargo build --release --no-default-features
	@echo "✅ Production build completed at: target/release/media-organizer"

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test --no-default-features

# Check code
check:
	@echo "🔍 Checking code..."
	cargo check --no-default-features

# Run linter
lint:
	@echo "📋 Running clippy linter..."
	cargo clippy --no-default-features -- -D warnings

# Format code
format:
	@echo "✨ Formatting code..."
	cargo fmt

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Run application
run:
	@echo "🏃 Running MediaOrganizer..."
	cargo run --no-default-features

# Complete build pipeline
all: format lint test dev
	@echo "✅ Complete build pipeline finished!"

# Install system dependencies (macOS)
install-deps-macos:
	@echo "📦 Installing macOS dependencies..."
	brew install pkg-config
	@echo "✅ macOS dependencies installed!"

# Install system dependencies (Ubuntu/Debian)
install-deps-ubuntu:
	@echo "📦 Installing Ubuntu/Debian dependencies..."
	sudo apt update
	sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.0-dev libssl-dev libsqlite3-dev
	@echo "✅ Ubuntu/Debian dependencies installed!"

# Cross-platform build targets
build-windows:
	@echo "🪟 Building for Windows..."
	cargo build --release --target x86_64-pc-windows-gnu --no-default-features

build-linux:
	@echo "🐧 Building for Linux..."
	cargo build --release --target x86_64-unknown-linux-gnu --no-default-features

build-macos:
	@echo "🍎 Building for macOS..."
	cargo build --release --target x86_64-apple-darwin --no-default-features

# Performance testing
bench:
	@echo "⚡ Running benchmarks..."
	cargo bench --no-default-features

# Documentation
docs:
	@echo "📚 Generating documentation..."
	cargo doc --no-default-features --open

# Default target
build: dev