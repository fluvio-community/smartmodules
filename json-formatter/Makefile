.SILENT:build test smdk-test clean

build:
	smdk build
	cargo build

test: build
	cargo test

smdk-test: build
	@smdk test --file ./test-data/input.txt -e spec='{ "match": [ { "key": "/type", "value": "subscribe", "format": { "with": ":loudspeaker: {} ({}) subscribed on {}", "using": [ "/name", "/email", "/source" ], "output": "/formatted" } }, { "key": "/type", "value": "use-case", "format": { "with": ":confetti_ball: {} ({}) wants to solve the following '{}' use-case:\n>{}", "using": [ "/name", "/email", "/source", "/description" ], "output": "/formatted" } } ], "default": { "format": { "with": "{} ({}) submitted a request", "using": [ "/name", "/email" ], "output": "/formatted" } } }'

clean:
	smdk clean
