.SILENT:build test smdk-test clean

build:
	smdk build
	cargo build

test: build
	cargo test

smdk-test: build
	@smdk test --file ./test-data/input.json --raw -e spec='[{"replace": {"regex": "\\d{3}-\\d{2}-\\d{4}", "with": "***-**-****" }},{"replace": {"regex": "(?P<first>\"address\":\\s+\")([\\w\\d\\s]+),", "with": "${first}..." }}]'

clean:
	smdk clean