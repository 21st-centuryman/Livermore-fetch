<div align="center">

# Livermore-fetch
##### Fetch, and process stock data!

[![Rust](https://img.shields.io/badge/Yahoo_finance_api-6001D2.svg?style=for-the-badge&logo=yahoo)](https://docs.rs/yahoo_finance_api/latest/yahoo_finance_api/)
[![Python](https://img.shields.io/badge/Polars-CD792C.svg?style=for-the-badge&logo=polars&logoColor=white)](https://www.pola.rs/)
</div>

## ⇁  Introduction
Welcome to Livermore fetch. 

The goal of this program is to fetch and process data from the yahoo finance api, then this will be processed using [polars](https://www.pola.rs/) to account for after market trading.

## ⇁  Structure
Currently Livermore-fetch is only one main.rs file, this might change in the future as the product grows.

## ⇁  Contribute
If you have any contribution feel free to add them I am more than inclined to make this work.

## ⇁  Current status
This is still a major work in progress, please see below for the 4 stages to this:

[X] 1: Fetch all symbols used in the yahoo api.
[X] 2: Fetch all the daily stock information for the last 10 years.
[] 3: Integrate this with [polars](https://www.pola.rs/) to account for after market trades.
[] 4: Add a dockerfile to make this able to be executed monthly on servers headlessly.
