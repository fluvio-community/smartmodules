## Parquet JSON Smartmodule

SmartModule that transforms a Parquet file into individual JSON Records. This SmartModule is [array_map] type, where each record-in generates a one or more records-out.

## Expected Input/Output

The file take a Parquet file as input and outputs a JSON record for each row in the Parquet file.

In our tests, the output will generate the followig JSON array:

```json
[
  {
    "am": 1,
    "carb": 4,
    "cyl": 6,
    "disp": 160,
    "drat": 3.9,
    "gear": 4,
    "hp": 110,
    "model": "Mazda RX4",
    "mpg": 21,
    "qsec": 16.46,
    "vs": 0,
    "wt": 2.62
  }
]
```

Checkout the full output file here: [output.json](./test-data/output.json).

## SMDK Compatible

This project works with `smdk` command tools:

```
smdk build
```

Test small file:

```
smdk test --file ./test-data/mtcars.parquet --raw
```


## Cargo Compatible

Build & Test

```
cargo build
```

```
cargo test
```


## Limitations

To use this Smartmodule, your parquet files must be chunked into files that can be use 1Mb of memory or less. This is because the parquet reader must be able to read the entire file into memory at once.

For files that are larger than 100Mb, you'll get the following error:

```bash
Requested memory 1012531200b exceeded max allowed 1000000000b
```

For chunking your parquet files, checkout the client script at: [https://github.com/fluvio-community/utilities](https://github.com/fluvio-community/utilities)


[array_map]: https://www.fluvio.io/docs/smartmodules/features/operators/array-map