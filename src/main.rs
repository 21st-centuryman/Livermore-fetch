use std::collections::HashSet;
use time::{ext::NumericalDuration, OffsetDateTime};
use tokio_test;
use yahoo_finance_api as yahoo;

fn main() {
    let now = OffsetDateTime::now_utc();
    let end = now.clone();
    let start = now.clone().saturating_sub(3652.days());

    let screener: HashSet<String> = screener();

    println!("{:?}", screener);
    println!("{:?}", screener.len());

    for stock in screener {
        fetch_api(&stock, start, end);
    }
}

fn screener() -> HashSet<String> {
    let provider = yahoo::YahooConnector::new();

    let mut screener: HashSet<String> = Default::default();

    for a in 65u8..97 {
        // Goes from A to (right before a)
        let value = (a as char).to_string();
        let resp = tokio_test::block_on(provider.search_ticker(&value)).unwrap();
        for item in resp.quotes {
            screener.insert(item.symbol);
        }
    }

    return screener;
}

fn fetch_api(stock: &str, start: OffsetDateTime, end: OffsetDateTime) {
    let provider = yahoo::YahooConnector::new();
    let resp = tokio_test::block_on(provider.get_quote_history(stock, start, end));

    match resp {
        Ok(resp) => {
            build_csv(stock, resp);
        }
        Err(_msg) => {
            println!("FAILED: {}", stock);
        }
    };
}

fn build_csv(name: &str, resp: yahoo_finance_api::YResponse) {
    for quote in resp.quotes().unwrap() {
        println!("{} | {:?} | {:?}", name, quote.open, quote.close);
    }
}
