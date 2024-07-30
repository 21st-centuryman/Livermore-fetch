<div align="center">

# Livermore-fetch
##### Fetch, and process stock data!
[![Docker](https://img.shields.io/badge/Docker-2496ED.svg?style=for-the-badge&logo=docker&logoColor=white)]()
[![Rust](https://img.shields.io/badge/Yahoo_finance_api-6001D2.svg?style=for-the-badge&logo=yahoo)]()
[![Python](https://img.shields.io/badge/Polars-CD792C.svg?style=for-the-badge&logo=polars&logoColor=white)]()
</div>

## ⇁  Introduction
Welcome to Livermore fetch. 

The goal of this program is to fetch and process data from the yahoo finance api, then this will be processed using [polars](https://www.pola.rs/) to account for after market trading.

## ⇁  Run
Livermore-fetchs can be run in two modes, `pull` and `process`. Below are the output of the help commands:
```console
$ Livermore-fetch -h   
fetching and processing stock data

Usage: Livermore-fetch <COMMAND>

Commands:
  pull, -P, --pull        Pull data from screener csv
  process, -p, --process  Process data for Livermore-analyze
  help                    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Then we also have help commands for the `pull` and `process` commands:
```console
$ Livermore-fetch --pull -h
Pull data from screener csv

Usage: Livermore-fetch {pull|--pull|-P} [path] [path]

Arguments:
  [path] [path]  /path/to/screener && /path/to/output/

Options:
  -h, --help  Print help
```
Pull will automatically append if the output file exists. It will also add fake data in between days as to mimick forex trading. Afterall this is my project.
Note that pull needs a csv with the first column having all ticker symbols. 
```console
$ Livermore-fetch --process -h
Process data for Livermore-analyze

Usage: Livermore-fetch {process|--process|-p} [path] [path]

Arguments:
  [path] [path]  /path/to/data && /path/to/output

Options:
  -h, --help  Print help
```
Process will make everything a single long column called *Tape*. 

We also have a `dockerfile` and a `docker-compose.yml` example. If you want to edit stuff like how often it is run you edit the `dockerfile`.
