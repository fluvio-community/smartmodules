.SILENT:build test-comma test-comma-readable test-semicolon-snake test-semicolon-snake-readable test-semicolon-camel test-semicolon-camel-readable

build:
	smdk build
	cargo build

test:
	cargo test

smdk-test: test-comma test-transit test-semicolon-snake test-semicolon-camel

test-comma: build
	@smdk test --file ./test-data/comma/input.csv --raw
	
test-transit: build
	@smdk test -e header_case=snake --file ./test-data/transit/input.csv --raw

test-semicolon-snake: build
	@smdk test -e delimiter=";" -e header_case=snake --file ./test-data/semicolon-snake/input.csv --raw

test-semicolon-camel: build
	@smdk test -e delimiter=";" -e header_case=camel --file ./test-data/semicolon-camel/input.csv --raw

make clean:
	@smdk clean