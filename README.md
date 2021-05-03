![main branch build](https://github.com/zeenix/transactions/actions/workflows/rust.yml/badge.svg?branch=main)

# Transactions

A simple transactions handler, written just for my personal amusement and learning. The main aim of
the exercise is to try out the `cvs` crate and compare it with other Serde implementations out
there. If this happens to be useful for any other purpose to anyone, they're free to use it.

## Usage

```sh
cargo run -- transactions.csv
```

where the `transactions.csv` should contain transaction records in CSV format. There are 5 types of
transaction records supported: deposit, withdrawal, dispute, resolve and chargeback. All the
transactions are processed and a final report on all known accounts is printed on the standard output.

Here is a sample input:

```csv
type,       client, tx, amount
deposit,    77,     1,  1.5
deposit,    80,     2,  2.0
withdrawal, 77,     3,  1.0
dispute,    77,     3,
chargeback, 77,     3,
chargeback, 80,     5,
```

```csv
client,available,held,total,locked
77,-0.5,0.0,-0.5,true
80,2.0,0.0,2.0,false
```

This will also result in a warning on the standard error output, since one of the chargeback
transaction is invalid:

```
Failed to process a transaction record: Missing referenced transaction record
```

## csv issues

While [`csv`](https://crates.io/crates/csv) seems like a nice API, the Serde implementations
seem a bit flaky. I encountered two issues so far:

* Internally-tagged enums are not only unsupported, you only find this out through a runtime error
  (and hence think that you're doing something wrong). While it seems the support
  [will not](https://github.com/BurntSushi/rust-csv/issues/211) (or rather can not) be added, on
  the maintainer's request I filed [a PR](https://github.com/BurntSushi/rust-csv/pull/231) to
  document this very unexpected limitation.
* Manually serializing a sequence of structs, we seem to somehow ends up with headers duplicated `n`
  times, where `n` is the number of structs. Perhaps another `csv` bug/limitation? Further
  investigation is needed.

## TODO

* Describe final accounts report.
* Check if `csv` deserializer works efficiently and not just loads everything upfront from the
  provided reader.
* More tests. There are already some essential integration tests but no unit tests.
* Accept CSV data from stdin, through a special `-` argument.
* Option to output accounts report in human-readable format.
* Struct serializing issue mentioned above.
