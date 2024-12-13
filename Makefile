.SILENT:build test smdk-test clean

# List of projects
PROJ = \
	array-map-json \
	csv-json-array \
	csv-json-records \
	json-formatter \
	key-gen-json \
	regex-json \
	regex-text \
	rss-json

all: build

build:
	@echo "Building projects..."
	@$(foreach proj, $(PROJ), $(MAKE) -C $(proj) build;)

test:
	@echo "Running tests in all projects..."
	@$(foreach proj, $(PROJ), $(MAKE) -C $(proj) test;)

smdk-test:
	@echo "Running smdk tests in all projects..."
	@$(foreach proj, $(PROJ), $(MAKE) -C $(proj) smdk-test;)

clean:
	@echo "Cleaning projects..."
	@$(foreach proj, $(PROJ), $(MAKE) -C $(proj) clean;)

.PHONY: all build test clean
