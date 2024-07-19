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
            if CsvReadOptions::default()
                .try_into_reader_with_file_path(Some(file.path()))
                .unwrap()
                .finish()
                .unwrap()
                .height()
                > 0
            {
                let mut tape = CsvReadOptions::default()
                    .try_into_reader_with_file_path(Some(file.path()))
                    .unwrap()
                    .finish()
                    .unwrap()
                    .drop("TIMESTAMP")
                    .expect("Can't remove TIMESTAMP")
                    .lazy()
                    .select([
                        col("CLOSE"),
                        concat_list([col("OPEN"), col("CLOSE")])
                            .expect("REASON")
                            .alias("TAPE"),
                    ])
                    .collect()
                    .expect("Can't create TAPE")
                    .select(["TAPE"])
                    .expect("Can't isolate TAPE")
                    .explode(["TAPE"])
                    .expect("Can't Explode Tape");

                CsvWriter::new(
                    File::create(format!(
                        "{}/{}",
                        path_to,
                        file.file_name().to_str().unwrap()
                    ))
                    .unwrap(),
                )
                .include_header(true)
                .with_separator(b',')
                .finish(&mut tape)
                .expect("Write failed");
            }
        });
}
