#![feature(type_alias_impl_trait)]
#![feature(iter_map_windows)]

use std::{array, env, fmt::Debug, fs, path::PathBuf};

use plotters::{
    chart::ChartBuilder,
    prelude::{IntoDrawingArea, IntoSegmentedCoord, SVGBackend},
    series::Histogram,
    style::{
        Color,
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
    let path = format!("{home}/ngrams");
    let mut ngrams: [_; 3] = get_ngrams(&path);
    let reference_ngrams = get_ngrams("eng_wiki_1m");

    unique_ngrams(&ngrams, None, "", false);
    unique_ngrams(&ngrams, None, " (increase)", true);
    num_per_ngram(&ngrams, None, "");

    unique_ngrams(&ngrams, Some(&reference_ngrams), " (referenced)", false);
    unique_ngrams(
        &ngrams,
        Some(&reference_ngrams),
        " (referenced + increase)",
        true,
    );
    num_per_ngram(&ngrams, Some(&reference_ngrams), " (referenced)");

    for (n, ngrams) in ngrams.iter_mut().enumerate() {
        let len = match n {
            0 => 80,
            1 => 115,
            2 => 90,
            _ => unreachable!(),
        };
        ngrams.truncate(len);
    }

    num_per_ngram(&ngrams, None, " (filtered)");
    num_per_ngram(&ngrams, Some(&reference_ngrams), " (filtered + referenced)");
}

fn get_ngrams(path: &str) -> [Vec<(u64, CustomDebugString)>; 3] {
    array::from_fn(|n| {
        let file = PathBuf::from(format!("{path}/{}-grams.txt", n + 1));
        let contents = fs::read_to_string(file).unwrap();
        let mut ngrams = contents
            .lines()
            .filter(|line| !line.is_empty())
            .flat_map(|entry| entry.split_once(' '))
            .flat_map(|(number, entry)| {
                Some((
                    number.parse::<u64>().ok()?,
                    CustomDebugString::cleaned(entry),
                ))
            })
            .collect::<Vec<_>>();

        ngrams.sort_by(|(num1, _), (num2, _)| num1.cmp(num2).reverse());
        ngrams
    })
}

fn unique_ngrams(
    ngrams: &[Vec<(u64, CustomDebugString)>; 3],
    reference: Option<&[Vec<(u64, CustomDebugString)>; 3]>,
    modifiers: &str,
    increase: bool,
) {
    let f_increases = |ngrams: &[Vec<(u64, CustomDebugString)>; 3]| {
        std::iter::once((1, 1))
            .chain(
                ngrams
                    .iter()
                    .enumerate()
                    .map_windows(|[(_, last), (n, current)]| (n + 1, current.len() / last.len())),
            )
            .collect::<Vec<_>>()
    };

    let increases = (f_increases(ngrams), reference.map(f_increases));

    let max = if increase {
        *increases
            .1
            .iter()
            .chain(&[increases.0.clone()])
            .flatten()
            .map(|(_, increase)| increase)
            .max()
            .unwrap()
    } else {
        ngrams
            .iter()
            .chain(reference.iter().flat_map(|ngrams| ngrams.iter()))
            .map(|ngram| ngram.len())
            .max()
            .unwrap()
    };

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

    for (i, (ngrams, color)) in reference
        .map(|ngrams| (ngrams, BLUE))
        .iter()
        .chain(&[(ngrams, RED)])
        .enumerate()
    {
        let color = color.mix(if reference.is_some() { 0.7 } else { 1. });

        chart
            .draw_series(
                Histogram::vertical(&chart)
                    .style(color.filled())
                    .margin(10)
                    .data(if increase {
                        if reference.is_some() && i == 0 {
                            increases.1.clone().unwrap()
                        } else {
                            increases.0.clone()
                        }
                    } else {
                        ngrams
                            .iter()
                            .enumerate()
                            .map(|(n, ngram)| (n + 1, ngram.len()))
                            .collect()
                    }),
            )
            .unwrap();
    }

    root.present().unwrap();
}

fn num_per_ngram(
    ngrams: &[Vec<(u64, CustomDebugString)>; 3],
    reference: Option<&[Vec<(u64, CustomDebugString)>; 3]>,
    modifiers: &str,
) {
    for n in 0..ngrams.len() {
        let f_max = |ngrams: &[Vec<(u64, CustomDebugString)>; 3]| {
            *ngrams[n].iter().map(|(num, _)| num).max().unwrap()
        };
        let maxes = (f_max(ngrams), reference.map(f_max));

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
            .build_cartesian_2d(segmented_coord, 0..(maxes.0 + maxes.0 / 20))
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

        for (i, (ngrams, color)) in reference
            .map(|ngrams| (ngrams, BLUE))
            .iter()
            .chain(&[(ngrams, RED)])
            .enumerate()
        {
            let color = color.mix(if reference.is_some() { 0.7 } else { 1. });

            // scale down reference so the maxes match
            let div = if reference.is_some() && i == 0 {
                maxes.1.unwrap() / maxes.0
            } else {
                1
            };

            chart
                .draw_series(
                    Histogram::vertical(&chart)
                        .style(color.filled())
                        .margin(0)
                        .data(
                            ngrams[n]
                                .iter()
                                .filter(|(_, ngram)| ngram.0 != "�")
                                .map(|(num, ngram)| (ngram, *num / div)),
                        ),
                )
                .unwrap();
        }

        root.present().unwrap();
    }
}
