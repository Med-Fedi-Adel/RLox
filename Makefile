.PHONY: run run-file build release check test clean

run:
	cargo run

run-file:
	cargo run -- $(FILE)

build:
	cargo build

release:
	cargo build --release

check:
	cargo check

test:
	cargo test

clean:
	cargo clean
