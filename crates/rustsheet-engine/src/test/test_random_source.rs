#![allow(clippy::unwrap_used)]
//! RAND, RANDBETWEEN and RANDARRAY draw from the model's random source (T-5100).

use crate::random_source::{RandomSource, LARGEST_DRAW};
use crate::test::util::new_empty_model;
use crate::Model;

const FORMULAS: [(&str, &str); 4] = [
    ("A1", "=RAND()"),
    ("A2", "=RANDBETWEEN(1,6)"),
    ("A3", "=RANDARRAY(1,2,5,9,TRUE)"),
    ("A4", "=RANDARRAY(1,1,10,20)"),
];

fn evaluated(source: RandomSource) -> Model {
    let mut model = new_empty_model();
    model.set_random_source(source);
    for (cell, formula) in FORMULAS {
        model.set_a1(cell, formula);
    }
    model.evaluate();
    model
}

fn texts(model: &Model) -> Vec<String> {
    ["A1", "A2", "A3", "B3", "A4"]
        .iter()
        .map(|cell| model.get_text_a1(cell))
        .collect()
}

#[test]
fn a_seeded_source_gives_the_same_values_on_every_run() {
    let first = texts(&evaluated(RandomSource::seeded(42)));
    assert_eq!(first, texts(&evaluated(RandomSource::seeded(42))));
    assert_ne!(first, texts(&evaluated(RandomSource::seeded(43))));
}

#[test]
fn setting_the_source_again_restarts_its_sequence() {
    let mut model = evaluated(RandomSource::seeded(5));
    let first = texts(&model);
    model.set_random_source(RandomSource::seeded(5));
    model.evaluate();
    assert_eq!(texts(&model), first);
    assert_eq!(model.random_source(), RandomSource::seeded(5));
}

#[test]
fn fixed_draws_give_the_ends_of_each_range() {
    let low = evaluated(RandomSource::fixed(0.0).unwrap());
    assert_eq!(texts(&low), ["0", "1", "5", "5", "10"]);
    let high = evaluated(RandomSource::fixed(LARGEST_DRAW).unwrap());
    assert_eq!(&texts(&high)[1..4], ["6", "9", "9"]);
}

#[test]
fn a_new_model_draws_from_the_system_source() {
    assert_eq!(new_empty_model().random_source(), RandomSource::system());
}
