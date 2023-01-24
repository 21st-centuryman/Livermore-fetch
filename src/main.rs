use std::error::Error;
use yahoo_finance::{history, Interval, Timestamped};

#[tokio::main]
async fn print(value: &str) {
    match history::retrieve_interval(value, Interval::_10y).await {
        Err(e) => println!("Failed to call Yahoo: {:?}", e),
        Ok(data) => {
            for bar in &data {
                println!(
                    "{} | {} | {} | {}",
                    value,
                    bar.datetime(),
                    bar.open,
                    bar.close
                )
            }
        }
    }
}

fn main() ->  Result<(), Box<dyn Error>>  {
    let mut rdr = csv::Reader::from_path("screener/nasdaq_screener.csv")?;
    for result in rdr.records() {
        let record = result?;
        print(record.get(0).unwrap());
    }
    Ok(())
}

