## Regex-Map JSON Smartmodule

SmartModule to read a JSON record, look-up values, run regex, and write the result back into the record. This SmartModule is [map] type, where each record-in generates a new records-out.

### Input Record

A JSON object:

```json
{
  "customer": {
    "first": "Abby",
    "last": "Hardy",
    "ssn": "123-45-6789"
  },
  "description": "Highlights: 43 Entity: Draft Documents [Encased string - (data)] (<a href='https://example.com/doc1/182031340621?pdf_header=&de_seq_num=44&caseid=456177'>9</a>)"
}
```

### Transformation spec

The transformation spec takes two types of `regex` operations: `capture` and `replace`. 

Regex `captures` retrieves a substring from a json value, and it requires the following parameters:

* `regex`: perl style regular expressions (as used by Rust Regex)
* `target`: the path notation of the `json` value we operate on
    * i.e top level: `/description`; nested: `/name/last` or `/names/1/last`
* `output`: the path of the `json` key for the output:
    * if the key exists, it is overwritten; otherwise it is created.
    * the path will inject into the json at various hierarcies

Regex `replace` replaces substrings in a json value, and it requires the following parameters:

* `regex`: perl style regular expressions (as used by Rust Regex)
* `target`: the path notation of the `json` value
* `with`: the string to replace the value matched by regex

In this example, we'll use the following transformation spec:

```yaml
transforms:
  - uses: fluvio/regex-json@0.1.0
    with:
      spec:
        - capture:
            regex: "(?i)Highlights:\\s+(\\w+)\\b"
            target: "/description"
            output: "/parsed/highlights"        
        - capture: 
            regex: "(?i)Entity:\\s+([\\w,\\s\\.\\']*\\S)\\s*\\["
            target: "/description"
            output: "/parsed/entity"
        - capture:
            regex: "href='([^']+)'"
            target: "/description"
            output: "/parsed/doc-link"
        - replace:
            regex: "\\d{3}-\\d{2}-\\d{4}"
            target: "/customer/ssn"
            with: "***-**-****"
```

### Outpot Record

A JSON object with a new `parsed` tree, and masked `ssn` value:

```json
{
  "customer": {
    "first": "Abby",
    "last": "Hardy",
    "ssn": "***-**-****"
  },
  "description": "Highlights: 43 Entity: Draft Documents [Encased string - (data)] (<a href='https://example.com/doc1/182031340621?pdf_header=&de_seq_num=44&caseid=456177'>9</a>)",
  "parsed": {
    "doc-link": "https://example.com/doc1/182031340621?pdf_header=&de_seq_num=44&caseid=456177",
    "entity": "Draft Documents",
    "highlights": "43"
  }
}
```

Note, no result is generated if the `target` key cannot be found, or the `regex` capture operation returns no matches.


### Build binary

Use `smdk` command tools to build:

```bash
smdk build
```

### Inline Test 

Use `smdk` to test:

```bash
smdk test --file ./test-data/input.json --raw -e spec='[{"capture": {"regex": "(?i)Highlights:\\s+(\\w+)\\b", "target": "/description", "output": "/parsed/highlights"}}, {"replace": {"regex": "\\d{3}-\\d{2}-\\d{4}", "target": "/customer/ssn", "with": "***-**-****" }}]'
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


[map]: https://www.fluvio.io/docs/smartmodules/features/operators/map
[Regex Docs]: https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html