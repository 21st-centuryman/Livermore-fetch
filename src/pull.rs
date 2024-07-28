use kdam::{Bar, TqdmIterator};
use polars::prelude::*;
use std::fs::File;
use tokio::runtime::Runtime;
use yahoo_finance_api::{self as yahoo, Quote};

pub fn pull(csv_file: &str, path_to: &str, pb: Bar) {
    CsvReader::new(std::io::BufReader::new(File::open(csv_file).unwrap())) // Stuff to read the file
        .finish()
        .unwrap()[0] // Get the first value, ie all ticker symbols
        .iter()
        .map(|item| item.to_string().replace("\"", "")) // Format and fix the dataframe
        .collect::<Vec<String>>()
        .iter()
        .tqdm_with_bar(pb) // Nice with a progress bar
        .for_each(|ticker| build_csv(ticker, get_quote_range(ticker), path_to))
}

fn get_quote_range(quote: &str) -> Vec<Quote> {
    match Runtime::new().unwrap().block_on(async {
        yahoo::YahooConnector::new()
            .unwrap()
            .get_quote_range(quote, "1d", "10y")
            .await
    }) {
        Ok(quotes) => quotes.quotes().unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn build_csv(name: &str, q: Vec<Quote>, path_to: &str) {
    if !q.is_empty() {
        let mut df = DataFrame::new(vec![
            Series::new(
                "TIMESTAMP",
                q.iter()
                    .take(q.len() - 1)
                    .map(|a| a.timestamp)
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "OPEN",
                q.iter()
                    .take(q.len() - 1)
                    .map(|a| a.open)
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "HIGH",
                q.iter()
                    .take(q.len() - 1)
                    .map(|a| a.high)
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "LOW",
                q.iter()
                    .take(q.len() - 1)
                    .map(|a| a.low)
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "VOLUME",
                q.iter()
                    .take(q.len() - 1)
                    .map(|a| a.volume)
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "CLOSE",
                q.iter()
                    .take(q.len() - 1)
                    .map(|a| a.close)
                    .collect::<Vec<_>>(),
            ),
            Series::new(
                "ADJCLOSE",
                q.iter()
                    .take(q.len() - 1)
                    .map(|a| a.adjclose)
                    .collect::<Vec<_>>(),
            ),
        ])
        .unwrap();

        let filepath = format!("{}/{}.csv", path_to, name.replace("/", "|"));
        CsvWriter::new(File::create(filepath).expect("Can't create file"))
            .include_header(true)
            .with_separator(b',')
            .finish(&mut df)
            .expect("Can't create file");
    }
}
