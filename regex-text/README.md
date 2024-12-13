## Regex-Text Smartmodule

SmartModule to read a record, run `replace-all` regex, and write the result back into the record. This SmartModule is [map] type, where each record-in generates a new records-out.

### Input Record

An arbitrary record (it does not need to be JSON):

```json
{
  "description": "Independence High School",
  "class": "2025-A",
  "students": [
    {
        "first": "Abby",
        "last": "Hardy",
        "ssn": "123-45-6789",
        "address": "285 LA PALA DR APT 2343, SAN JOSE CA 95127"
    },
    {
        "first": "Bob",
        "last": "Newmal",
        "ssn": "987-65-4321",
        "address": "21 E TRIMBLE RD, Santa Clara CA 95347"
    },
    {
        "first": "Cindy",
        "last": "Hall",
        "ssn": "999-888-7777",
        "address": "1601 PRIME PL, Milpitas CA 95344"
    }
  ]
}
```

### Transformation spec

The transformation spec defines a list of `replace` regex operations:

* `regex`: perl style regular expressions (as used by Rust Regex)
* `with`: the string to replace the value matched by regex

In this example, we'll use the following transformation spec:

```yaml
transforms:
  - uses: fluvio/regex-text@0.1.0
    with:
      spec:
        - replace:
            regex: "\\d{3}-\\d{2}-\\d{4}"
            with: "***-**-****"
        - replace:
            regex: "(?P<first>\"address\":\\s+\")([\\w\\d\\s]+),"
            with: "${first}..."
```

### Outpot Record

A new record with the output of the transformations:

```json
{
  "class": "2025-A",
  "description": "Independence High School",
  "students": [
    {
      "address": "... SAN JOSE CA 95127",
      "first": "Abby",
      "last": "Hardy",
      "ssn": "***-**-****"
    },
    {
      "address": "... Santa Clara CA 95347",
      "first": "Bob",
      "last": "Newmal",
      "ssn": "***-**-****"
    },
    {
      "address": "... Milpitas CA 95344",
      "first": "Cindy",
      "last": "Hall",
      "ssn": "***-**-****"
    }
  ]
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
smdk test --file ./test-data/input.json --raw -e spec='[{"replace": {"regex": "\\d{3}-\\d{2}-\\d{4}", "with": "***-**-****" }},{"replace": {"regex": "(?P<first>\"address\":\\s+\")([\\w\\d\\s]+),", "with": "${first}..." }}]'
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

### References

* [Regex Docs]


[map]: https://www.fluvio.io/docs/smartmodules/features/
[Regex Docs]: https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html