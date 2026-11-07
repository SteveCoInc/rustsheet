#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// HOUR, MINUTE and SECOND round the time of day to the nearest second, carrying into the minute
// and the hour (`time_of_day_parts`; probes/second-rounding, T-1408).
#[test]
fn time_parts_round_to_the_nearest_second() {
    let mut model = new_empty_model();
    // 12:29:59.99997 -> 12:30:00.000
    model.set_a1("A1", "=MINUTE(0.520833333)");
    model.set_a1("A2", "=SECOND(0.520833333)");
    // TIME drops the fraction of its seconds: still 45
    model.set_a1("A3", "=SECOND(TIME(14,30,45.999))");
    // 00:00:00.9996 -> 00:00:01
    model.set_a1("A4", "=SECOND(0.9996/86400)");
    // A time that rounds up to midnight
    model.set_a1("A5", "=MINUTE(0.99999999999)");
    // 10:59:59.6 carries into the hour (probes/second-rounding A15)
    model.set_a1("A6", "=HOUR(TIME(10,59,59)+0.6/86400)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "30");
    assert_eq!(model.get_text_a1("A2"), "0");
    assert_eq!(model.get_text_a1("A3"), "45");
    assert_eq!(model.get_text_a1("A4"), "1");
    assert_eq!(model.get_text_a1("A5"), "0");
    assert_eq!(model.get_text_a1("A6"), "11");
}

// YEAR, MONTH, DAY and WEEKDAY carry past midnight where HOUR rolls to 0, and every date and time
// part is #NUM! from fl(2958466-0.5/86400) up (`carried_day`; probes/t3137-date-parts-near-midnight).
#[test]
fn date_parts_carry_past_midnight() {
    let mut model = new_empty_model();
    let edge: f64 = 1.0 - 0.5 / 86400.0;
    let below = f64::from_bits(edge.to_bits() - 1);
    model.set_a1("A1", &format!("=DAY({edge:?})"));
    model.set_a1("A2", &format!("=DAY({below:?})"));
    model.set_a1("A3", "=YEAR(2958466-0.5/86400)");
    model.set_a1("A4", "=HOUR(2958466-0.5/86400)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("A2"), "0");
    assert_eq!(model.get_text_a1("A3"), "#NUM!");
    assert_eq!(model.get_text_a1("A4"), "#NUM!");
}
