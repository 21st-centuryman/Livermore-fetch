use chrono::prelude::*;
use polars::prelude::*;
use std::collections::HashSet;
use std::fs::File;
use tokio_test;
use yahoo_finance_api::{self as yahoo, Quote};

fn main() {
    search_tickers()
        .iter()
        .for_each(|ticker| build_csv(ticker, add_fake_vals(get_quote_range(ticker))));
}

fn search_tickers() -> HashSet<String> {
    let mut results: HashSet<String> = Default::default();
    results.extend((65u8..97).flat_map(|a| {
        tokio_test::block_on(
            yahoo::YahooConnector::new()
                .unwrap()
                .search_ticker(&(a as char).to_string()),
        )
        .unwrap()
        .quotes
        .iter()
        .map(|sym| sym.symbol.clone())
        .collect::<Vec<_>>()
    }));
    return results;
}

fn get_quote_range(quote: &str) -> Vec<Quote> {
    match tokio_test::block_on(
        yahoo::YahooConnector::new()
            .unwrap()
            .get_quote_range(quote, "1d", "10y"),
    ) {
        Ok(quotes) => quotes.quotes().unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/*
I have this function to reduce noise in the data, by making fake data, I basically want to eliminate days off. Next project I'll just use forex.

I am not planning to make this AI actually trade stocks. For that I need more data and if I were to write  a function that collects this date
I would never be allowed a finance job. Since my IP would be blocked everywhere and possible legal action could be taken.

This is the nasdaq finance API and we are working with UTC. (I should write a blog post for my hate towards timezones, UTC everywhere!!!)

Anyway with this info we can draw a nice little line between a fake value and two real values

      -------------------------------------------------------------------------
      ^              ^             ^           ^              ^               ^
    close d1   midnight d1    open fake     close fake    midnight fake    open d2
UTC: 20:00        24:00         13:30        20:00           24:00           13:30

We basically split the day into midnight and not, so we check the trend and make two values, midnight d1 and midnight fake. then we split if we need
more days.
*/
fn add_fake_vals(quotes: Vec<Quote>) -> Vec<Vec<f64>> {
    let mut new_quotes: Vec<Vec<f64>> = vec![];
    let mut old_q: &Quote = if !quotes.is_empty() {
        &quotes[0]
    } else {
        return vec![];
    };
    for quote in &quotes {
        let days_since = ((quote.clone().timestamp - old_q.timestamp) / 86400) as i32;
        if &quotes[0] != quote || days_since.clone() != 1 {
            let total = 24.0 * days_since as f32 + 4.0 + 13.5;
            let price_diff = quote.open.clone() as f32 - old_q.close.clone() as f32;
            let midnight1 = old_q.close.clone() as f32 + (4.0 / total) * price_diff;
            let midnight2 = quote.open.clone() as f32 - ((total - 13.5) / total) * price_diff;
            let midnights: Vec<f32> = (0..days_since)
                .map(|i: i32| {
                    midnight1 + (i as f32 * (midnight2 - midnight1) / (days_since - 1) as f32)
                })
                .collect();
            for i in 1..days_since {
                new_quotes.push(vec![
                    (i * 86400 + old_q.timestamp.clone() as i32) as f64,
                    midnights[i as usize - 1] as f64
                        + 0.0965 * (midnights[i as usize] - midnights[i as usize - 1]) as f64,
                    midnights[i as usize - 1] as f64
                        + 0.675 * (midnights[i as usize] - midnights[i as usize - 1]) as f64,
                ]);
            }
        }
        new_quotes.push(vec![
            quote.timestamp.clone() as f64,
            quote.open.clone() as f64,
            quote.close.clone() as f64,
        ]);
        old_q = quote;
    }
    return new_quotes;
}

fn build_csv(name: &str, quotes: Vec<Vec<f64>>) {
    let mut df = DataFrame::new(vec![
        Series::new(
            "TIMESTAMP",
            quotes
                .iter()
                .map(|a| {
                    DateTime::from_timestamp(a[0] as i64, 0)
                        .unwrap()
                        //.format("%Y-%m-%d ")
                        .to_string()
                })
                .collect::<Vec<_>>(),
        ),
        Series::new(
            "OPEN",
            quotes
                .iter()
                .map(|a| format!("{:.3}", a[1]))
                .collect::<Vec<_>>(),
        ),
        Series::new(
            "CLOSE",
            quotes
                .iter()
                .map(|a| format!("{:.3}", a[2]))
                .collect::<Vec<_>>(),
        ),
    ])
    .unwrap();

    CsvWriter::new(File::create(format!("./data/{name}.csv")).expect("could not create file"))
        .include_header(false)
        .with_separator(b',')
        .finish(&mut df);
    println!("Ticker: {name} DONE");
}
