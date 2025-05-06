use std::{array, env, fs, path::PathBuf};

use plotters::{
    chart::ChartBuilder,
    prelude::{BitMapBackend, IntoDrawingArea},
    series::LineSeries,
    style::full_palette::{RED, WHITE},
};

fn main() {
    // get n-grams
    let home = env::var("HOME").unwrap();
    let ngrams: [_; 3] = array::from_fn(|n| {
        let file = PathBuf::from(format!("{home}/ngrams/{}-grams.txt", n + 1));
        let contents = fs::read_to_string(file).unwrap();
        contents
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(|entry| entry.split_once(' ').unwrap())
            .map(|(number, entry)| (number.parse::<u32>().unwrap(), entry.to_string()))
            .collect::<Vec<_>>()
    });

    // setup plot
    let root = BitMapBackend::new("target/out.png", (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .build_cartesian_2d(0..3usize, 0..20_000usize)
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            ngrams
                .into_iter()
                .enumerate()
                .map(|(n, ngram)| (n, ngram.len())),
            &RED,
        ))
        .unwrap();

    root.present().unwrap();
}
