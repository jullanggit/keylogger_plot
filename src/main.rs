use std::{array, env, fmt::Debug, fs, path::PathBuf};

use plotters::{
    chart::ChartBuilder,
    prelude::{IntoDrawingArea, IntoSegmentedCoord, SVGBackend},
    series::Histogram,
    style::{
        Color, ShapeStyle,
        full_palette::{BLUE, RED, WHITE},
    },
};

#[derive(Clone, PartialEq)]
struct CustomDebugString(String);
impl CustomDebugString {
    fn cleaned(str: &str) -> Self {
        Self(
            str.char_indices()
                .map(|(position, char)| match char {
                    '\0' => "\\0",
                    '\u{1}' => "^A", // heading
                    '\u{3}' => "^C",
                    '\u{8}' => "\\b", // backspace
                    '\u{9}' => "\\t", // tab
                    '\u{12}' => "^R", // device-control 2
                    '\u{14}' => "^T", // device-control 4
                    '\u{16}' => "^V",
                    '\u{17}' => "^W",
                    '\u{18}' => "^X",  // cancel
                    '\u{1B}' => "\\e", // escape
                    '\u{7F}' => "\\d", // delete
                    ' ' | '\u{A0}' => "\\s",
                    other if other.is_control() => "�",
                    other => &str[position..(position + other.len_utf8())],
                })
                .collect::<String>(),
        )
    }
}
impl Debug for CustomDebugString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
                    CustomDebugString::cleaned(entry),
                )
            })
            .collect::<Vec<_>>();

        ngrams.sort_by(|(num1, _), (num2, _)| num1.cmp(num2).reverse());
        ngrams
    });

    let keylogger_style = RED.mix(0.5).filled();
    let reference_style = BLUE.mix(0.5).filled();

    unique_ngrams(&ngrams, "", keylogger_style);
    num_per_ngram(&ngrams, "", keylogger_style);

    for (n, ngrams) in ngrams.iter_mut().enumerate() {
        let len = match n {
            0 => 80,
            1 => 115,
            2 => 90,
            _ => unreachable!(),
        };
        ngrams.truncate(len);
    }

    unique_ngrams(&ngrams, " (filtered)", keylogger_style);
    num_per_ngram(&ngrams, " (filtered)", keylogger_style);
}

fn unique_ngrams(
    ngrams: &[Vec<(u32, CustomDebugString)>; 3],
    modifiers: &str,
    style: impl Into<ShapeStyle> + Clone,
) {
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

fn num_per_ngram(
    ngrams: &[Vec<(u32, CustomDebugString)>; 3],
    modifiers: &str,
    style: impl Into<ShapeStyle> + Clone,
) {
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
                    .style(style.clone())
                    .margin(0)
                    .data(
                        ngrams[n]
                            .iter()
                            .filter(|(_, ngram)| ngram.0 != "�")
                            .map(|(num, ngram)| (ngram, *num)),
                    ),
            )
            .unwrap();

        root.present().unwrap();
    }
}
