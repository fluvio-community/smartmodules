.SILENT:build test smdk-test clean

build:
	smdk build
	cargo build

test: build
	cargo test

smdk-test: build
	@smdk test --file ./test-data/input.json --raw -e spec='[{"capture": {"regex": "(?i)Highlights:\\s+(\\w+)\\b", "target": "/description", "output": "/parsed/highlights"}}, {"replace": {"regex": "\\d{3}-\\d{2}-\\d{4}", "target": "/customer/ssn", "with": "***-**-****" }}]'

clean:
	smdk clean
