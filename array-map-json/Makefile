.SILENT:build test smdk-test clean

build:
	smdk build
	cargo build

test: build
	cargo test

smdk-test: build
	@smdk test --file ./test-data/input.json --raw

clean:
	smdk clean