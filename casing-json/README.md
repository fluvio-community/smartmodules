## Casing JSON Smartmodule

SmartModule to read convert JSON keys casing. This SmartModule is [map] type, where each record-in generates a new records-out.

### Input Record

A JSON object:

```json
{
  "fooBar": {
    "moreFooBar": "foo"
  }
}
```

### Transformation spec

The transformation spec accepts two keywords/params:

- `casing`: what case should the key be converted to (`snake`, `camel`, `pascal`, `kebab`, `constant`, `cobol`)
  - default is `snake`
- `depth`: how deep in the JSON tree should it go
  - default is [u8::MAX](https://doc.rust-lang.org/std/u8/constant.MAX.html)

In this example, we'll use the following transformation spec:

```yaml
transforms:
  - uses: fluvio/casing-json@0.1.0
    with:
      spec:
        casing: kebab
        depth: 1
```

### Outpot Record

The same input JSON object, but with its keys renamed (only first level since we set the `depth` to `1`):

```json
{
  "FOO-BAR": {
    "MORE-FOO-BAR": "foo"
  }
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
smdk test --file ./test-data/input.json --raw -e spec="{\"casing\":\"camel\"}"
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
