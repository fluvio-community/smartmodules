## Key-Gen JSON Smartmodule

SmartModule to read values from JSON Records, compute a digest, and update record key. This SmartModule is [map] type, where each record-in generates a new records-out.

### Input Record

A JSON object:

```json
{
  "description": "This is the description of my JSON object",
  "last_build_date": "Tue, 18 Apr 2023 15:00:01 GMT",
  "link": "http://www.example.com",
  "pub_date": "Mon, 17 Apr 2023 16:08:23 GMT",
  "title": "My Json Object Title"
}
```

### Transformation spec

The transformation spec requires one keyword:
* `lookup`: an array of elements that allows you to choose the `json values` to generate the digest.
    * For nested values, use path notation such as `/name/last` or `/names/1/last`
    * If you want the full nested tree to be used, just use the root: `/name`.

In this example, we'll use the following transformation spec:

```yaml
  transforms:
    - uses: fluvio/key-gen-json@0.1.0
      with:
        spec:
          lookup:
            - "/pub_date"
            - "/last_build_date"
          key_name: "dedup_key"
```

### Outpot Record

A JSON object augmented with `dedup_key`, and a digest:

```json
{
  "dedup_key": "3193200642d322d171dd4c05875741ff7a4fc0f7a467b52d514d5ce273d4f762",
  "description": "This is the description of my JSON object",
  "last_build_date": "Tue, 18 Apr 2023 15:00:01 GMT",
  "link": "http://www.example.com",
  "pub_date": "Mon, 17 Apr 2023 16:08:23 GMT",
  "title": "My Json Object Title"
}
```

### Build binary

Use `smdk` command tools to build:

```bash
smdk build
```

### Inline Test 

Use `smdk` to test:

```bash
smdk test --file ./test-data/input.json --raw -e spec="{\"lookup\":[\"\/pub_date\", \"\/last_build_date\"], \"key_name\": \"dedup_key\"}"
```

### Cluster Test

Use `smdk` to load to cluster:

```bash
smdk load 
```

Test using `transform.yaml` file:

```bash
smdk test --file ./test-data/input.json --raw  --transforms-file ./test-data/transform.yaml
```

### Cargo Compatible

Build & Test

```
cargo build
```

```
cargo test
```


[map]: https://www.fluvio.io/docs/smartmodules/features/operators/map
