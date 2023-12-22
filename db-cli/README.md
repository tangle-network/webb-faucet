## How to use the CLI

`db-cli` is a command line interface for the database. It reads the database and output all the accounts in a JSON format.

**Usage:**
```bash
Usage: db-cli -d <db> [-v <verbosity>] [-e <evm-output>] [-s <substrate-output>]

Webb Faucet Database CLI

Options:
  -d, --db          sled database path
  -v, --verbosity   control verbosity level
  -e, --evm-output  output file for evm addresses
  -s, --substrate-output
                    output file for substrate addresses
  --help            display usage information
```

## Examples

1. Output all the accounts in the database to stderr
```bash
./db-cli -d ./faucet
```
2. Output all the accounts in the database to json files

```bash
./db-cli -d ./faucet -v 2 -e evm.json -s substrate.json
```
