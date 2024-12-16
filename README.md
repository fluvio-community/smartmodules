# Smartmodules

This repository contains a collection of smartmodules that can be used with the Fluvio CLI or inside Fluvio Connectors.

| Smartmodule Project           | Input   | Output | Description                           |
| ----------------------------- | ------- | ------ | ------------------------------------- |
| [rss-json]                    | xml     | json   | Parses RSS XML input into JSON format |
| [json-formatter]              | json    | json   | Generated a formatted string from JSON values |
| [key-gen-json]                | json    | json   | Generates a unique key (digest) from JSON values |
| [array-map-json]              | json    | json   | Splits an JSON array into individual records |
| [regex-json]                  | json    | json   | Applies Regex transformations on JSON values |
| [regex-text]                  | text    | text   | Applies Regex transformations on arbitrary text |
| [csv-json-array]              | csv     | json   | Turns a CVS file into an array of json records | 
| [csv-json-records]            | csv     | json   | Turns a CVS file into individual of json records | 
| [parquet-json-records]        | parquet | json   | Turns a parquet file into individual of json records | 


## Download from Hub

These smartmodules have been published in the Hub. 

List the smartmodules:

```bash
fluvio hub smartmodule list
```

Download to your cluster:

```bash
fluvio hub smartmodule download <smarmodule-name>
```

List the smartmodules on your cluster:

```bash
fluvio smartmodule list
```

Use smartmodules in the consumer:

```bash
fluvio consume <topic> --smartmodule <smartmodule-name> 
```

## Compile & Run

Use the readme in for the specific smartmodule for instructions on how to compile and run.


## Develop Your Smartmodule

Checkout the documentation on how to [build your smartmodule].



[rss-json]: rss-json/README.md
[json-formatter]: json-formatter/README.md
[key-gen-json]: key-gen-json/README.md
[array-map-json]: array-map-json/README.md
[regex-json]: regex-json/README.md
[regex-text]: regex-text/README.md
[csv-json-array]: csv-json-array/README.md
[csv-json-records]: csv-json-records/README.md
[parquet-json-records]: parquet-json-records/README.md

[build your smartmodule]: https://www.fluvio.io/docs/smartmodules/developers/overview