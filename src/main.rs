use chrono::{Datelike, TimeZone, Utc};
use tokio_test;
use yahoo_finance_api as yahoo;

fn main() {
    let provider = yahoo::YahooConnector::new();

    let now = Utc::now();
    let start = Utc
        .with_ymd_and_hms(now.year() - 10, now.month(), now.day(), 0, 0, 0)
        .unwrap();
    let end = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap();

    let stock = "AAPL";
    let resp = tokio_test::block_on(provider.get_quote_history(stock, start, end)).unwrap();
    let quotes = resp.quotes().unwrap();
    let mut index = 0;
    for quote in quotes {
        println!(
            "{}: | {} | {} | {} | {} |",
            index, stock, quote.timestamp, quote.open, quote.close
        );
        index += 1;
    }

    //let resp = tokio_test::block_on(provider.search_ticker("`")).unwrap();

    //println!("All tickers found while searching:");
    //for item in resp.quotes {
    //    println!("{}", item.symbol)
    //}

    println!("{:?}", screener())

}


fn screener() -> Vec<String> {
    let provider = yahoo::YahooConnector::new();

    let mut screener: Vec<String> = Default::default();

    for a in 65u8..97 { // Goes from A to (right before a)
       let value = (a as char).to_string();
        let resp = tokio_test::block_on(provider.search_ticker(&value)).unwrap();
        for item in resp.quotes {
            screener.push(item.symbol);
        }
     }

    return screener;
}
