use chrono::{DateTime, Datelike, TimeZone, Utc};
use tokio_test;
use yahoo_finance_api as yahoo;

fn main() {

    let now = Utc::now();
    let start = Utc
        .with_ymd_and_hms(now.year() - 10, now.month(), now.day(), 0, 0, 0)
        .unwrap();
    let end = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap();

    let screener: Vec<String> = screener();

    for stock in screener {
        fetch_api(&stock, start, end);
    }
}

fn screener() -> Vec<String> {
    let provider = yahoo::YahooConnector::new();

    let mut screener: Vec<String> = Default::default();

    for a in 65u8..97 {
        // Goes from A to (right before a)
        let value = (a as char).to_string();
        let resp = tokio_test::block_on(provider.search_ticker(&value)).unwrap();
        for item in resp.quotes {
            screener.push(item.symbol);
        }
    }

    return screener;
}

fn fetch_api(stock: &str, start: DateTime<Utc>, end: DateTime<Utc>) {
    let provider = yahoo::YahooConnector::new();
    let resp = tokio_test::block_on(provider.get_quote_history(stock, start, end));
    let _resp = match resp.unwrap().quotes() {
        Ok(_resp) => println!("Success: {}", stock),
        Err(msg) => println!("Stock. {}. Failed: {}", stock, msg),
    };


    // 
    // COMMENT
    // MAKE SURE THE FAILURE AT FETCHING DATA DOES NOT LEAD TO THE END OF THE EXECUTION
    //
}
