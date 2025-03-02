BINARY_NAME = studfinder
CARGO_FLAGS = --release

.PHONY: all build test clean install dev lint docs

all: clean build test lint

build:
	cargo build $(CARGO_FLAGS)

test:
	cargo test

clean:
	cargo clean
	rm -f *.db
	rm -rf target/

install:
	cargo install --path .

dev:
	cargo watch -x run

lint:
	cargo clippy -- -D warnings
	cargo fmt --check

format:
	cargo fmt

lint-fix:
	cargo clippy --fix -- -D warnings
	cargo fmt

lint-all: lint
	cargo clippy --all-targets --all-features -- -D warnings
	cargo doc --no-deps --document-private-items --warn-missing-docs
	@echo "All linting checks passed!"

docs:
	cargo doc --no-deps

db-setup:
	cargo run -- init

release:
	cargo build --release
	@echo "Binary located at target/release/$(BINARY_NAME)"

check: lint test
	cargo audit
