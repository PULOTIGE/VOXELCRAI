.PHONY: build release clean test run

build:
	cargo build

release:
	cargo build --release

clean:
	cargo clean

test:
	cargo test

run:
	cargo run --release

# Bare-metal AArch64 build
build-arm:
	cargo build --target aarch64-unknown-none --release

# Install cross-compilation target
install-arm-target:
	rustup target add aarch64-unknown-none
