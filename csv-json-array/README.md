## CSV to JSON Array Smartmodule

SmartModule to convert a Comman Separated Values (CSV) file into a JSON array, where each CSV row is a JSON record. It handles configurable delimiters and several header types. This SmartModule is [map] type, where each record-in generates a new records-out.


### CSV File

Given the following CSV file:

```csv
Username; Identifier;First name;Last name
booker12;9012;Rachel;Booker
grey07;2070;Laura;Grey
johnson81;4081;Craig;Johnson
jenkins46;9346;Mary;Jenkins
smith79;5079;Jamie;Smith
```

### Expected Result

The Smartmodule produces the following result:

```json
[
  {
    "first_name": "Rachel",
    "identifier": "9012",
    "last_name": "Booker",
    "username": "booker12"
  },
  {
    "first_name": "Laura",
    "identifier": "2070",
    "last_name": "Grey",
    "username": "grey07"
  },
  {
    "first_name": "Craig",
    "identifier": "4081",
    "last_name": "Johnson",
    "username": "johnson81"
  },
  {
    "first_name": "Mary",
    "identifier": "9346",
    "last_name": "Jenkins",
    "username": "jenkins46"
  },
  {
    "first_name": "Jamie",
    "identifier": "5079",
    "last_name": "Smith",
    "username": "smith79"
  }
]
```

### SmartModule Parameters

The SmartModule offers parameters to handle delimiters and headers:

- `delimiter`: The delimiter used in the CSV file. Default is `,`.
- `header_case`: The case of the header. Default is `none`. Possible values are `snake`, `camel`, `none`.

### Build binary

Use `smdk` command tools to build:

```bash
smdk build
```

### Inline Test 

In another terminal:

```bash
$ smdk test -e delimiter=";" -e header_case=snake --file ./test-data/semicolon-snake/input.csv --raw
```

### Cargo Compatible

Build & Test

```
cargo build
```

```
cargo test
```


# Tests

* For additional tests, checkout the [Makefile](./Makefile)


[map]: https://www.fluvio.io/docs/smartmodules/features/