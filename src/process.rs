use kdam::{Bar, TqdmIterator};
use polars::prelude::*;
use std::fs::{DirEntry, File};

pub fn process(path: &str, path_to: &str, pb: Bar) {
    std::fs::read_dir(path)
        .unwrap()
        .filter_map(|file| file.ok())
        .filter(|file| file.path().extension().unwrap_or_default() == "csv")
        .collect::<Vec<_>>()
        .iter()
        .tqdm_with_bar(pb)
        .for_each(|file: &DirEntry| {
            if !(CsvReadOptions::default()
                .try_into_reader_with_file_path(Some(file.path()))
                .unwrap()
                .finish()
                .unwrap()
                .is_empty())
            {
                let mut tape = CsvReadOptions::default()
                    .try_into_reader_with_file_path(Some(file.path()))
                    .unwrap()
                    .finish()
                    .unwrap()
                    .drop("HIGH")
                    .expect("Can't remove HIGH")
                    .drop("LOW")
                    .expect("Can't remove LOW")
                    .drop("VOLUME")
                    .expect("Can't remove VOLUME")
                    .drop("ADJCLOSE")
                    .expect("Can't remove ADJCLOSE");

                tape = add_fake_vals(tape);
                if !tape.is_empty() {
                    tape = tape
                        .lazy()
                        .select([
                            col("CLOSE"),
                            concat_list([col("OPEN"), col("CLOSE")]).expect("REASON").alias("TAPE"),
                        ])
                        .collect()
                        .expect("Can't create TAPE")
                        .select(["TAPE"])
                        .expect("Can't isolate TAPE")
                        .explode(["TAPE"])
                        .expect("Can't Explode Tape");

                    CsvWriter::new(File::create(format!("{}/{}", path_to, file.file_name().to_str().unwrap())).unwrap())
                        .include_header(true)
                        .with_separator(b',')
                        .finish(&mut tape)
                        .expect("Write failed");
                }
            }
        });
}

/*
I have this function to reduce noise in the data, by making fake data, I basically want to eliminate days off. Next project I'll just use forex.

I am not planning to make this AI actually trade stocks. For that I need more data and if I were to write  a function that collects this date
I would never be allowed a finance job. Since my IP would be blocked everywhere and possible legal action could be taken.

This is the nasdaq finance API and we are working with UTC. (I should write a blog post for my hate towards timezones, UTC everywhere!!!)

Anyway with this info we can draw a nice little line between a fake value and two real values

      -------------------------------------------------------------------------
      ^              ^             ^           ^              ^               ^
   close 1     midnight 1     open fake    close fake    midnight fake      open 2
UTC: 20:00        24:00         13:30        20:00           24:00           13:30

We basically split the day into midnight and not, so we check the trend and make two values, midnight 1 and midnight fake. then we split if we need
more days.
*/

#[derive(Debug, PartialEq)]
pub struct Quote {
    time: u64,
    open: f32,
    close: f32,
}

fn add_fake_vals(table: DataFrame) -> DataFrame {
    if table.height() < 2 {
        return Default::default();
    }

    let quotes: Vec<Quote> = table
        .clone()
        .into_struct("StructChunked")
        .iter()
        .map(|row| Quote {
            time: row[0].try_extract().unwrap(),
            open: row[1].try_extract().unwrap(),
            close: row[2].try_extract().unwrap(),
        })
        .collect();

    let mut old_q: &Quote = &quotes[0];
    let mut new_vals: Vec<Vec<f32>> = vec![];

    for quote in &quotes {
        let days_since = ((quote.time - old_q.time) / 86400) as i32;
        if &quotes[0] != quote || &days_since != &1 {
            let total = 24.0 * days_since as f32 + 4.0 + 13.5;
            let price_diff = quote.open.clone() as f32 - old_q.close.clone() as f32;
            let midnight1 = old_q.close.clone() as f32 + (4.0 / total) * price_diff;
            let midnight2 = quote.open.clone() as f32 - ((total - 13.5) / total) * price_diff;
            let midnights: Vec<f32> = (0..days_since)
                .map(|i: i32| midnight1 + (i as f32 * (midnight2 - midnight1) / (days_since - 1) as f32))
                .collect();
            for i in 1..days_since {
                new_vals.push(vec![
                    midnights[i as usize - 1] as f32 + 0.0965 * (midnights[i as usize] - midnights[i as usize - 1]) as f32,
                    midnights[i as usize - 1] as f32 + 0.675 * (midnights[i as usize] - midnights[i as usize - 1]) as f32,
                ]);
            }
        }
        new_vals.push(vec![quote.open.clone() as f32, quote.close.clone() as f32]);
        old_q = quote;
    }

    return DataFrame::new(vec![
        Series::new("OPEN", new_vals.iter().map(|a| a[0]).collect::<Vec<f32>>()),
        Series::new("CLOSE", new_vals.iter().map(|a| a[1]).collect::<Vec<f32>>()),
    ])
    .unwrap();
}
