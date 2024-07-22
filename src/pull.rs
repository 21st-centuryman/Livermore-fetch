use chrono::prelude::*;
use kdam::{Bar, TqdmIterator};
use polars::prelude::*;
use std::fs::File;
use std::path::Path;
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
        .for_each(|ticker| build_csv(ticker, add_fake_vals(get_quote_range(ticker)), path_to))
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
fn add_fake_vals(quotes: Vec<Quote>) -> Vec<Vec<f32>> {
    let mut new_quotes: Vec<Vec<f32>> = vec![];
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
                    (i * 86400 + old_q.timestamp.clone() as i32) as f32,
                    midnights[i as usize - 1] as f32
                        + 0.0965 * (midnights[i as usize] - midnights[i as usize - 1]) as f32,
                    midnights[i as usize - 1] as f32
                        + 0.675 * (midnights[i as usize] - midnights[i as usize - 1]) as f32,
                ]);
            }
        }
        new_quotes.push(vec![
            quote.timestamp.clone() as f32,
            quote.open.clone() as f32,
            quote.close.clone() as f32,
        ]);
        old_q = quote;
    }
    return new_quotes;
}

fn build_csv(name: &str, quotes: Vec<Vec<f32>>, path_to: &str) {
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
                .map(|a| format!("{:.3}", a[1]).parse::<f32>().unwrap())
                .collect::<Vec<_>>(),
        ),
        Series::new(
            "CLOSE",
            quotes
                .iter()
                .map(|a| format!("{:.3}", a[1]).parse::<f32>().unwrap())
                .collect::<Vec<_>>(),
        ),
    ])
    .unwrap();

    let filepath = format!("{}/{}.csv", path_to, name.replace("/", "|"));
    if Path::new(&filepath).exists()
        && CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(Path::new(&filepath).to_path_buf()))
            .unwrap()
            .finish()
            .unwrap()
            .height()
            > 1
        && df.height() > 1
    {
        let old_df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(Path::new(&filepath).to_path_buf()))
            .unwrap()
            .finish()
            .unwrap()
            .lazy()
            .select([
                col("TIMESTAMP").cast(DataType::String),
                col("OPEN").cast(DataType::Float32),
                col("CLOSE").cast(DataType::Float32),
            ])
            .collect()
            .expect("Can't cast datepoints");

        df = old_df
            .head(Some(old_df.height() - 1))
            .vstack(
                &df.tail(Some(
                    df.height()
                        - df.column("TIMESTAMP")
                            .expect("Can't select TIMESTAMP")
                            .iter()
                            .position(|x| {
                                &x == &old_df
                                    .tail(Some(2))
                                    .column("TIMESTAMP")
                                    .unwrap()
                                    .iter()
                                    .next()
                                    .unwrap()
                            })
                            .unwrap()
                        - 1,
                )),
            )
            .expect("Can't concatenate");
    }
    CsvWriter::new(File::create(filepath).expect("Can't create file"))
        .include_header(true)
        .with_separator(b',')
        .finish(&mut df)
        .expect("Can't create file");
}
