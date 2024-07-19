use kdam::{Bar, TqdmIterator};
use polars::frame::row::*;
use polars::prelude::*;
use std::fs::{DirEntry, File};

pub fn process(path: &str, path_to: &str, size: usize, pb: Bar) {
    std::fs::read_dir(path)
        .unwrap()
        .filter_map(|file| file.ok())
        .filter(|file| file.path().extension().unwrap_or_default() == "csv")
        .for_each(|file: DirEntry| {
            if CsvReadOptions::default()
                .try_into_reader_with_file_path(Some(file.path()))
                .unwrap()
                .finish()
                .unwrap()
                .height()
                > size / 2
            {
                let tape = CsvReadOptions::default()
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
                let mut dh: LazyFrame = Default::default();
                (0..(tape.height() - size))
                    .tqdm_with_bar(pb.clone())
                    .for_each(|i: usize| {
                        let seq_vals = (i..(i + size + 1))
                            .map(|val: usize| tape.get_row(val).unwrap().0.get(0).unwrap().clone())
                            .collect::<Vec<_>>();
                        let new_dh = DataFrame::from_rows(&[Row::new(seq_vals)]).unwrap();

                        dh = concat([dh.clone(), new_dh.clone().lazy()], UnionArgs::default())
                            .unwrap();
                    });
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
                .finish(&mut dh.collect().unwrap())
                .expect("Write failed");
            }
        });
}
