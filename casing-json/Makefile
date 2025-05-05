.SILENT:build test smdk-test test-casing-default test-casing-depth test-casing-array test-casing-camel test-casing-pascal test-casing-kebab test-casing-const test-casing-cobol clean

build:
	smdk build
	cargo build

test: build
	cargo test

smdk-test: test-casing-default test-casing-depth test-casing-array test-casing-camel test-casing-pascal test-casing-kebab test-casing-const test-casing-cobol

test-casing-default: build
	@smdk test --file ./test-data/input.json --raw

test-casing-depth: build
	@smdk test --file ./test-data/input.json --raw --params spec='{"depth": 1}'

test-casing-array: build
	@smdk test --file ./test-data/input-array.json --raw --params spec='{"depth": 3}'

test-casing-camel: build
	@smdk test --file ./test-data/input.json --raw --params spec='{"casing": "camel"}'

test-casing-pascal: build
	@smdk test --file ./test-data/input.json --raw --params spec='{"casing": "pascal"}'

test-casing-kebab: build
	@smdk test --file ./test-data/input.json --raw --params spec='{"casing": "kebab"}'

test-casing-const: build
	@smdk test --file ./test-data/input.json --raw --params spec='{"casing": "constant"}'

test-casing-cobol: build
	@smdk test --file ./test-data/input.json --raw --params spec='{"casing": "cobol"}'

clean:
	smdk clean