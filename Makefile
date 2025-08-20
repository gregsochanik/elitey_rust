.PHONY: build test run clean lint check format help

# Default target
all: build

# Build the project
build:
	cargo build

# Build release version
release:
	cargo build --release

# Run tests
test:
	cargo test

# Run the game
run:
	cargo run

# Run the game in release mode
run-release:
	cargo run --release

# Clean build artifacts
clean:
	cargo clean

# Run linter
lint:
	cargo clippy -- -D warnings

# Check if code compiles without warnings
check:
	cargo check

# Format code
format:
	cargo fmt

# Run all checks (lint, test, check)
all-checks: lint test check

# Show help
help:
	@echo "Available targets:"
	@echo "  build        - Build the project"
	@echo "  release      - Build release version"
	@echo "  test         - Run tests"
	@echo "  run          - Run the game"
	@echo "  run-release  - Run the game in release mode"
	@echo "  clean        - Clean build artifacts"
	@echo "  lint         - Run linter"
	@echo "  check        - Check if code compiles"
	@echo "  format       - Format code"
	@echo "  all-checks   - Run lint, test, and check"
	@echo "  help         - Show this help"
