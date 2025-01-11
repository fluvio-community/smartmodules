## Solana Balance Calculator

Balance Calculator Smartmodule computes balance changes in a  EncodedTransactionWithStatusMeta record. If no balance change is detected, the record is ignored.

## Expected Input/Outpot

The input is a EncodedTransactionWithStatusMeta record. Checkout the sample input file [./test-data/input-record.json](./test-data/input-record.json).

The output will generate the followig result:

```json
[
  {
    "account": "2ATdozUDANVdw1um7Lf82bZ4hKPtMGMT4pH42CrLKfn6",
    "preBalance": 7600182,
    "postBalance": 1703981890,
    "difference": 1696381708
  },
  {
    "account": "47kUcJY97j4argbJveNAwFGt3mK8vvSDm4e5vcawFk3B",
    "preBalance": 619005615169,
    "postBalance": 617308697333,
    "difference": -1696917836
  }
]
```

## Build & Test with SMDK

Use `smdk` command tools to build and test the smartmodule:

```
smdk build
```

Test using the sample record:

```
smdk test --file test-data/input-record.json --raw
```

## Cargo Compatible

Build & Test

```
cargo build
```

```
cargo test
```

## InfinyOn Hub

This smartmodule is available in the [InfinyOn Hub](https://infinyon.cloud/ui/hub).

```bash
fluvio/solana-balance-calculator@0.1.0
```

## Test on Fluvio

Load the smartmodule to Fluvio:

```bash
fluvio hub smartmodule download fluvio/solana-balance-calculator@0.1.0
```

Create a topic & produce the sample record:

```bash
fluvio topic create solana
fluvio produce solana --file test-data/input-record.json --raw
```

Consume using the smartmodule:

```bash
fluvio consume solana --smartmodule fluvio/solana-balance-calculator@0.1.0 -Bd --output json
```
