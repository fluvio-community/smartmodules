## JSON Formatter Smartmodule

SmartModule to read values from JSON records, generate a user-define formatted string, and update the record. This SmartModule is [map] type, where each record-in generates a new records-out.

### Input Records

Each row is an individual JSON record:

```json
{
  "email": "alice@acme.com",
  "name": "Alice Liddell",
  "type": "subscribe",
  "source": "front-page"
}
{
  "email": "bob@acme.com",
  "name": "Bob Newland",
  "type": "use-case",
  "source": "iot",
  "description": "Tracking fleet of trucks"
}
{
  "email": "charlie@acme.com",
  "name": "Charlie Brimmer",
  "type": "use-case",
  "source": "clickstream",
  "description": "Track user interests"
}
{
  "email": "random@acme.com",
  "name": "Random Dude",
  "type": "undefined"
}
```

### Transformation spec

The transformation spec covers 2 cases:
* `match`: a list of formatting rules associated with a match condition.
* `default`: a default formatter without a matching rule. 

The `default` formatter is required and can be used with or without the `match` section.
If used together, the matching statement is checked first, and default is applied if no matching record is found.
 

In this example, we'll use the following transformation spec:

```yaml
spec:
  match:
    - key: "/type"
      value: "subscribe"
      format:
        with: ":loudspeaker: {} ({}) subscribed on {}"
        using: 
          - "/name"
          - "/email"
          - "/source"
        output: "/formatted"
    - key: "/type"
      value: "use-case"
      format:
        with: ":confetti_ball: {} ({}) wants to solve the following '{}' use-case:\n>{}"
        using: 
          - "/name"
          - "/email"
          - "/source"
          - "/description"
        output: "/formatted"
  default:
    format: 
      with: "{} ({}) submitted a request"
      using: 
        - "/name"
        - "/email"
      output: "/formatted"
```

NOTE: The JSON object references for look-up and output must be in [JSON Pointer] notation.

### Output Records

JSON records augmented with `formatted` strings:

```json
{
  "email": "alice@acme.com",
  "formatted": ":loudspeaker: Alice Liddell (alice@acme.com) subscribed on front-page",
  "name": "Alice Liddell",
  "source": "front-page",
  "type": "subscribe"
}
{
  "description": "Tracking fleet of trucks",
  "email": "bob@acme.com",
  "formatted": ":confetti_ball: Bob Newland (bob@acme.com) wants to solve the following 'iot' use-case:\n>Tracking fleet of trucks",
  "name": "Bob Newland",
  "source": "iot",
  "type": "use-case"
}
{
  "description": "Track user interests",
  "email": "charlie@acme.com",
  "formatted": ":confetti_ball: Charlie Brimmer (charlie@acme.com) wants to solve the following 'clickstream' use-case:\n>Track user interests",
  "name": "Charlie Brimmer",
  "source": "clickstream",
  "type": "use-case"
}
{
  "email": "random@acme.com",
  "formatted": "Random Dude (random@acme.com) submitted a request",
  "name": "Random Dude",
  "type": "undefined"
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
smdk test --file ./test-data/input.txt -e spec='
 { "match": [ { "key": "/type", "value": "subscribe", "format": { "with": ":loudspeaker: {} ({}) subscribed on {}", "using": [ "/name", "/email", "/source" ], "output": "/formatted" } }, { "key": "/type", "value": "use-case", "format": { "with": ":confetti_ball: {} ({}) wants to solve the following '{}' use-case:\n>{}", "using": [ "/name", "/email", "/source", "/description" ], "output": "/formatted" } } ], "default": { "format": { "with": "{} ({}) submitted a request", "using": [ "/name", "/email" ], "output": "/formatted" } } }'
```

### Cluster Test

Use `smdk` to load the startmodule to a fluvio cluster:

```bash
smdk load 
```

Test using `transform.yaml` file:

```bash
smdk test --file ./test-data/input.txt  --transforms-file ./test-data/transform.yaml
```


NOTE: Changing the parameters inside the `transforms` files yield different results.


### Cargo Compatible

The project has tests that can be run using `cargo`:

```
cargo build
```

```
cargo test
```


[map]: https://www.fluvio.io/docs/smartmodules/features/operators/map
[JSON Pointer]: https://datatracker.ietf.org/doc/html/rfc6901