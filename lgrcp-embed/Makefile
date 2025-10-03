prepare:
	rustup target add thumbv7em-none-eabihf
	rustup component add llvm-tools-preview
	cargo install cargo-binutils

run:
	cargo clean
	cargo build --release --target thumbv7m-none-eabi
	cargo run

size:
	cargo clean
	cargo build
	cargo size -- -Ax
