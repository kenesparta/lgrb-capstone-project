prepare:
	rustup target add thumbv7m-none-eabi
	cargo install cargo-binutils
	rustup component add llvm-tools-preview

run:
	cargo clean
	cargo build --release --target thumbv7m-none-eabi
	cargo run
