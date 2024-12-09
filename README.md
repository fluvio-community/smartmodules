# Smartmodules

This repository contains a collection of smartmodules that can be used with the Fluvio CLI or inside Fluvio Connectors.

| Smartmodule Project           | Input  | Output | Description                           |
| ----------------------------- | ------ | ------ | ------------------------------------ |
| [rss-json]                    | xml    | json   | Parses RSS XML input into JSON format |
| [json-formatter]              | json   | json   | Generated a formatted string from JSON values |
| [key-gen-json]                | json   | json   | Generates a unique key (digest) from JSON values |
| [array-map-json]              | json   | json   | Splits an JSON array into individual records |
| [regex-map-json]              | json   | json   | Applies Regex transformations on JSON values |
| [regex-map]                   | text   | text   | Applies Regex transformations on arbitrary text |
| [csv-json-array]              | csv    | json   | Turns a CVS file into an array of json records | 
| [csv-json-records]            | csv    | json   | Turns a CVS file into individual of json records | 


## How to use smartmodules

You can build and run smartmodules locally, or you can download them from the Fluvio Hub.

* Run `fluvio hub smartmodules list` to see the ones available for download.

[rss-json]: rss-json/README.md
[json-formatter]: json-formatter/README.md
[key-gen-json]: key-gen-json/README.md
[array-map-json]: array-map-json/README.md
[regex-map-json]: regex-map-json/README.md
[regex-map]: regex-map/README.md
[csv-json-array]: csv-json-array/README.md
[csv-json-records]: csv-json-records/README.md