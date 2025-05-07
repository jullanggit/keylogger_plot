use std::{array, char, env, fmt::Debug, fs, path::PathBuf};

use plotters::{
    chart::ChartBuilder,
    prelude::{IntoDrawingArea, IntoSegmentedCoord, SVGBackend},
    series::Histogram,
    style::{
        Color,
        full_palette::{RED, WHITE},
    },
};

#[derive(Clone, PartialEq)]
struct CustomDebugString(String);
impl Debug for CustomDebugString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cleaned = self
            .0
            .replace("\u{8}", "\\b")
            .replace("\u{9}", "\\t")
            .replace("\u{1B}", "\\e")
            .replace("\u{7F}", "\\d");
        println!("{}", cleaned.escape_default());
        write!(f, "{}", cleaned.escape_default())
    }
}

fn main() {
    // get n-grams
    let home = env::var("HOME").unwrap();
    let mut ngrams: [_; 3] = array::from_fn(|n| {
        let file = PathBuf::from(format!("{home}/ngrams/{}-grams.txt", n + 1));
        let contents = fs::read_to_string(file).unwrap();
        let mut ngrams = contents
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(|entry| entry.split_once(' ').unwrap())
            .map(|(number, entry)| {
                (
                    number.parse::<u32>().unwrap(),
                    CustomDebugString(entry.to_string()),
                )
            })
            .collect::<Vec<_>>();

        ngrams.sort_by(|(num1, _), (num2, _)| num1.cmp(num2).reverse());
        ngrams
    });

    unique_ngrams(&ngrams, "");
    num_per_ngram(&ngrams, "");

    for ngrams in &mut ngrams {
        ngrams.retain(|(num, _)| *num > 10)
    }

    unique_ngrams(&ngrams, " (filtered)");
    num_per_ngram(&ngrams, " (filtered)");
}

fn unique_ngrams(ngrams: &[Vec<(u32, CustomDebugString)>; 3], modifiers: &str) {
    let max = ngrams.iter().map(|ngram| ngram.len()).max().unwrap();

    let path = format!("target/Unique N-Grams{modifiers}.svg");
    // setup plot
    let root = SVGBackend::new(&path, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .margin(10)
        .build_cartesian_2d((1..3usize).into_segmented(), 0..(max + max / 10))
        .unwrap();

    chart
        .configure_mesh()
        .disable_x_mesh()
        .x_desc("N-Gram length")
        .y_desc("Unique N-Grams")
        .axis_desc_style(("sans-serif", 20))
        .draw()
        .unwrap();

    chart
        .draw_series(
            Histogram::vertical(&chart)
                .style(RED.filled())
                .margin(10)
                .data(
                    ngrams
                        .iter()
                        .enumerate()
                        .map(|(n, ngram)| (n + 1, ngram.len())),
                ),
        )
        .unwrap();

    root.present().unwrap();
}

fn num_per_ngram(ngrams: &[Vec<(u32, CustomDebugString)>; 3], modifiers: &str) {
    for n in 0..ngrams.len() {
        let max = ngrams[n].iter().map(|(num, _)| num).max().unwrap();

        let path = format!("target/{}-grams{modifiers}.svg", n + 1);

        // setup plot
        let root = SVGBackend::new(&path, (1920, 1080)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let only_ngrams = ngrams[n]
            .iter()
            .map(|(_, ngram)| ngram.clone())
            .collect::<Vec<CustomDebugString>>();

        let segmented_coord = only_ngrams.as_slice().into_segmented();

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .margin(1)
            .build_cartesian_2d(segmented_coord, 0..(max + max / 10))
            .unwrap();

        chart
            .configure_mesh()
            .disable_x_mesh()
            .x_desc("N-Gram")
            .y_desc("Occurrences")
            .x_labels(usize::MAX)
            .axis_desc_style(("sans-serif", 20))
            .draw()
            .unwrap();

        chart
            .draw_series(
                Histogram::vertical(&chart)
                    .style(RED.filled())
                    .margin(0)
                    .data(ngrams[n].iter().map(|(num, ngram)| (ngram, *num))),
            )
            .unwrap();

        root.present().unwrap();
    }
}
