prepare:
	rustup target add thumbv7em-none-eabihf
	cargo install cargo-binutils
	rustup component add llvm-tools-preview
