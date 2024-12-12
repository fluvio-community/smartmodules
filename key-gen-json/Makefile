build:
	smdk build

test: build
	@smdk test --file ./test-data/input.json --raw -e spec="{\"lookup\":[\"\/pub_date\", \"\/last_build_date\"], \"key_name\": \"dedup_key\"}"

clean:
	@smdk clean
