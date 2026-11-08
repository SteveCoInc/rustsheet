#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! The filters against the probes' sidecars: each pivot is the case file's definition (the
//! shared probe harness, `pivot::probe_cases`) with the `<filters>` of its `xml` typed by hand as
//! the importer types them (the importer's own reading is tested in
//! crates/rustsheet-xlsx/src/import/pivots/tests.rs, and the eval harness runs every probe
//! through it), and every expectation is a cell Excel saved.

use super::*;
use crate::pivot::probe_cases::{
    probe_model, probe_pivot, probe_pivot_over, read, sheet, sidecar_differences,
};
use crate::pivot::{compute, refresh, CacheSource, MAX_PIVOT_WORK, MAX_RECORD_VALUES};

use FilterOperator::{Equal, GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual, NotEqual};
use PivotFilterKind::{Count, Percent, Sum};

fn filter(field: u32, kind: PivotFilterKind, criteria: FilterCriteria) -> PivotFilter {
    PivotFilter {
        field,
        kind,
        measure: 0,
        eval_order: -1,
        id: 1,
        criteria,
        whole_day: false,
    }
}

/// One `customFilter` per `(operator, val)`, all of them when `and`.
fn custom(and: bool, conditions: &[(FilterOperator, &str)]) -> FilterCriteria {
    FilterCriteria::Custom {
        and,
        conditions: conditions
            .iter()
            .map(|&(operator, value)| FilterCondition {
                operator,
                value: value.to_string(),
            })
            .collect(),
    }
}

fn caption(field: u32, conditions: &[(FilterOperator, &str)], and: bool) -> PivotFilter {
    filter(field, PivotFilterKind::Caption, custom(and, conditions))
}

fn value(field: u32, conditions: &[(FilterOperator, &str)], and: bool) -> PivotFilter {
    filter(field, PivotFilterKind::Value, custom(and, conditions))
}

fn top10(field: u32, kind: PivotFilterKind, top: bool, val: &str) -> PivotFilter {
    filter(
        field,
        kind,
        FilterCriteria::Top10 {
            top,
            value: val.to_string(),
        },
    )
}

/// Sheet `name` of probe `stem` with `filters`, against its sidecar.
fn differences(stem: &str, name: &str, filters: Vec<PivotFilter>) -> Vec<String> {
    let case = read(stem, "case");
    let (cache, mut table) = probe_pivot(&case, name);
    table.filters = filters;
    sidecar_differences(stem, name, &cache, &table, None, &[])
}

fn check(stem: &str, sheets: Vec<(&str, Vec<PivotFilter>)>) {
    let wrong: Vec<String> = sheets
        .into_iter()
        .flat_map(|(name, filters)| differences(stem, name, filters))
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn caption_filters_match_wildcards_and_ignore_case() {
    let p = 1;
    let both = |low: &str, high: &str| {
        caption(
            p,
            &[(GreaterThanOrEqual, low), (LessThanOrEqual, high)],
            true,
        )
    };
    check(
        "t4130-pivot-label-filters",
        vec![
            ("CEq", vec![caption(p, &[(Equal, "Cherry")], false)]),
            ("CNe", vec![caption(p, &[(NotEqual, "Cherry")], false)]),
            ("CBw", vec![caption(p, &[(Equal, "Ap*")], false)]),
            ("CNb", vec![caption(p, &[(NotEqual, "Ap*")], false)]),
            ("CEw", vec![caption(p, &[(Equal, "*e")], false)]),
            ("CNew", vec![caption(p, &[(NotEqual, "*e")], false)]),
            ("CCo", vec![caption(p, &[(Equal, "*an*")], false)]),
            ("CNc", vec![caption(p, &[(NotEqual, "*an*")], false)]),
            ("CGt", vec![caption(p, &[(GreaterThan, "Banana")], false)]),
            (
                "CGe",
                vec![caption(p, &[(GreaterThanOrEqual, "Banana")], false)],
            ),
            ("CLt", vec![caption(p, &[(LessThan, "Banana")], false)]),
            (
                "CLe",
                vec![caption(p, &[(LessThanOrEqual, "Banana")], false)],
            ),
            ("CBt", vec![both("B", "D")]),
            (
                "CNbt",
                vec![caption(p, &[(LessThan, "B"), (GreaterThan, "D")], false)],
            ),
            // The autoFilter decides, not stringValue1 (Cherry).
            ("CMix", vec![caption(p, &[(Equal, "Grape")], false)]),
            ("CBwU", vec![caption(p, &[(Equal, "AP*")], false)]),
            ("CCoU", vec![caption(p, &[(Equal, "*AN*")], false)]),
            ("CEqL", vec![caption(p, &[(Equal, "banana")], false)]),
            ("CGtU", vec![caption(p, &[(GreaterThan, "BANANA")], false)]),
            ("CBtI", vec![both("Banana", "Cherry")]),
        ],
    );
}

#[test]
fn value_filters_and_top_n_compare_the_measure() {
    let p = 1;
    let mut on_count = value(p, &[(GreaterThan, "1")], false);
    on_count.measure = 1;
    let both = |low: &str, high: &str| {
        value(
            p,
            &[(GreaterThanOrEqual, low), (LessThanOrEqual, high)],
            true,
        )
    };
    check(
        "t4130-pivot-value-filters",
        vec![
            ("VEq", vec![value(p, &[(Equal, "12")], false)]),
            ("VNe", vec![value(p, &[(NotEqual, "12")], false)]),
            ("VGt", vec![value(p, &[(GreaterThan, "11")], false)]),
            ("VGe", vec![value(p, &[(GreaterThanOrEqual, "11")], false)]),
            ("VLt", vec![value(p, &[(LessThan, "11")], false)]),
            ("VLe", vec![value(p, &[(LessThanOrEqual, "11")], false)]),
            ("VBt", vec![both("6", "12")]),
            (
                "VNbt",
                vec![value(p, &[(LessThan, "6"), (GreaterThan, "12")], false)],
            ),
            ("TC", vec![top10(p, Count, true, "2")]),
            ("TB", vec![top10(p, Count, false, "2")]),
            ("TP", vec![top10(p, Percent, true, "50")]),
            ("TS", vec![top10(p, Sum, true, "40")]),
            ("TCv", vec![top10(p, Count, true, "2")]),
            ("VM1", vec![on_count]),
        ],
    );
}

#[test]
fn top_n_keeps_count_ties_and_stops_percent_and_sum_at_the_threshold() {
    let p = 0;
    check(
        "t4164-pivot-top-n",
        vec![
            ("TC2", vec![top10(p, Count, true, "2")]),
            ("TB1", vec![top10(p, Count, false, "1")]),
            ("TC9", vec![top10(p, Count, true, "9")]),
            ("TP40", vec![top10(p, Percent, true, "40")]),
            ("TPb5", vec![top10(p, Percent, false, "5")]),
            ("TS50", vec![top10(p, Sum, true, "50")]),
            ("TS60", vec![top10(p, Sum, true, "60")]),
            ("TS110", vec![top10(p, Sum, true, "110")]),
            ("TSb10", vec![top10(p, Sum, false, "10")]),
            // The `<items>` order breaks a percent or sum tie, not the cache's.
            ("TP40i", vec![top10(p, Percent, true, "40")]),
            ("TP40o", vec![top10(p, Percent, true, "40")]),
            ("TSb10i", vec![top10(p, Sum, false, "10")]),
            ("TSb10o", vec![top10(p, Sum, false, "10")]),
            ("NT3", vec![top10(p, Count, true, "3")]),
            ("NB2", vec![top10(p, Count, false, "2")]),
            ("NP50", vec![top10(p, Percent, true, "50")]),
            ("NBP50", vec![top10(p, Percent, false, "50")]),
            ("NS3", vec![top10(p, Sum, true, "3")]),
            ("NBS", vec![top10(p, Sum, false, "10")]),
        ],
    );
}

#[test]
fn label_filters_and_hidden_items_come_before_top_n_and_value_filters_keep_their_order() {
    let p = 0;
    let not_apple = || caption(p, &[(NotEqual, "Apple")], false);
    let below = || value(p, &[(LessThan, "45")], false);
    let top2 = || top10(p, Count, true, "2");
    let half = || top10(p, Percent, true, "50");
    check(
        "t4164-pivot-filter-order",
        vec![
            ("HT", vec![top2()]),
            ("HP", vec![half()]),
            ("LT1", vec![not_apple(), top2()]),
            ("TL1", vec![top2(), not_apple()]),
            ("LP1", vec![not_apple(), half()]),
            ("PL1", vec![half(), not_apple()]),
            ("VT", vec![below(), top2()]),
            ("TV1", vec![top2(), below()]),
        ],
    );
}

#[test]
fn inner_field_filters_rank_per_parent_and_totals_cover_what_is_kept() {
    check(
        "t4164-pivot-filter-totals",
        vec![
            ("ITop", vec![top10(1, Count, true, "1")]),
            ("IVal", vec![value(1, &[(GreaterThan, "5")], false)]),
            ("IPct", vec![top10(1, Percent, true, "50")]),
            ("OTop", vec![top10(0, Count, true, "1")]),
            ("SH1L", vec![caption(1, &[(NotEqual, "A")], false)]),
        ],
    );
}

#[test]
fn the_other_criteria_and_two_fields_filter_as_excel_does() {
    let cherry = || caption(1, &[(Equal, "Cherry")], false);
    let values = FilterCriteria::Values {
        values: vec!["cherry".to_string()],
        blank: true,
        dates: Vec::new(),
    };
    let day = FilterCriteria::Values {
        values: Vec::new(),
        blank: false,
        dates: vec![DateGroupItem {
            year: 2024,
            month: 4,
            day: 17,
            hour: 0,
            minute: 0,
            second: 0,
            parts: 3,
        }],
    };
    let kind = PivotFilterKind::Caption;
    check(
        "t4130-pivot-filter-attrs",
        vec![
            ("FIcon", vec![filter(1, kind, FilterCriteria::Nothing)]),
            ("FVals", vec![filter(1, kind, values)]),
            ("FDgi", vec![filter(1, kind, day)]),
            ("FBtn", vec![cherry()]),
            ("FEo", vec![cherry()]),
            (
                "FMf1",
                vec![
                    caption(1, &[(NotEqual, "Apple")], false),
                    value(1, &[(GreaterThan, "5")], false),
                ],
            ),
            (
                "FTwo",
                vec![
                    caption(0, &[(Equal, "West")], false),
                    caption(1, &[(NotEqual, "Apple")], false),
                ],
            ),
        ],
    );
}

#[test]
fn auto_show_is_a_count_top_n() {
    let stem = "t4130-pivot-field-filters";
    let case = read(stem, "case");
    for (name, top) in [("AS", true), ("ASb", false)] {
        let (cache, mut table) = probe_pivot(&case, name);
        table.fields[1].auto_show = Some(AutoShow {
            count: 2,
            top,
            measure: 0,
        });
        let wrong = sidecar_differences(stem, name, &cache, &table, None, &[]);
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }
}

/// Sheet `name` of probe `stem` over the cache the engine's refresh reads from the case's cells
/// once they are calculated (its sources hold DATE formulas), with `filters`. The probe model has no date formats, so the dates are number items, which the filters
/// read alike.
fn calculated_differences(stem: &str, name: &str, filters: Vec<PivotFilter>) -> Vec<String> {
    let case = read(stem, "case");
    let mut model = probe_model(&case);
    model.evaluate();
    let spec = &sheet(&case, name)["pivot_tables"][0];
    let (sheet_name, range) = spec["source"].as_str().unwrap().split_once('!').unwrap();
    let empty = PivotCache {
        source: Some(CacheSource {
            sheet_name: sheet_name.into(),
            range: range.into(),
        }),
        ..PivotCache::default()
    };
    let cache = refresh(&empty, &model, MAX_RECORD_VALUES).unwrap();
    let (cache, mut table) = probe_pivot_over(&case, name, cache);
    table.filters = filters;
    sidecar_differences(stem, name, &cache, &table, None, &[])
}

fn date(conditions: &[(FilterOperator, &str)], and: bool, whole_day: bool) -> PivotFilter {
    let mut filter = filter(0, PivotFilterKind::Date, custom(and, conditions));
    filter.whole_day = whole_day;
    filter
}

/// Every cell of each of `sheets`, date item labels included: the check reads values, and a
/// date item's label is its serial in Excel's cells too (its text, `m/d/yyyy`, is T-4807's,
/// probes/t4717-pivot-cell-formats L5).
fn check_calculated(stem: &str, sheets: &[(String, PivotFilter)]) {
    let wrong: Vec<String> = sheets
        .iter()
        .flat_map(|(name, filter)| calculated_differences(stem, name, vec![filter.clone()]))
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn date_filters_compare_date_times_unless_use_whole_day() {
    let between = [
        (GreaterThanOrEqual, "45399.25"),
        (LessThanOrEqual, "45399.5"),
    ];
    let sheets = [
        ("WEqN", date(&[(Equal, "45399")], false, false)),
        ("WEq0", date(&[(Equal, "45399")], false, false)),
        ("WEq1", date(&[(Equal, "45399")], false, true)),
        ("WEqT0", date(&[(Equal, "45399.5")], false, false)),
        ("WEqT1", date(&[(Equal, "45399.5")], false, true)),
        ("WNt0", date(&[(GreaterThan, "45399")], false, false)),
        ("WNt1", date(&[(GreaterThan, "45399")], false, true)),
        ("WOe0", date(&[(LessThanOrEqual, "45399")], false, false)),
        ("WOe1", date(&[(LessThanOrEqual, "45399")], false, true)),
        ("WBt0", date(&between, true, false)),
        ("WBt1", date(&between, true, true)),
    ]
    .map(|(name, filter)| (name.to_string(), filter));
    check_calculated("t4164-pivot-date-times", &sheets);
}

#[test]
fn month_quarter_and_fixed_date_filters_keep_their_dates() {
    let dynamic = |name: &str| {
        filter(
            0,
            PivotFilterKind::Date,
            FilterCriteria::Dynamic(DynamicDate::from_name(name)),
        )
    };
    let between = [(GreaterThanOrEqual, "45399"), (LessThanOrEqual, "45492")];
    let mut sheets: Vec<(String, PivotFilter)> = [
        ("DEq", date(&[(Equal, "45399")], false, false)),
        ("DNe", date(&[(NotEqual, "45399")], false, false)),
        ("DOld", date(&[(LessThan, "45399")], false, false)),
        ("DOldEq", date(&[(LessThanOrEqual, "45399")], false, false)),
        ("DNt", date(&[(GreaterThan, "45492")], false, false)),
        ("DNte", date(&[(GreaterThanOrEqual, "45492")], false, false)),
        ("DBt", date(&between, true, false)),
        (
            "DNbt",
            date(&[(LessThan, "45399"), (GreaterThan, "45492")], false, false),
        ),
        ("DWd", date(&[(Equal, "45399")], false, true)),
    ]
    .map(|(name, filter)| (name.to_string(), filter))
    .into();
    for period in (1..=12)
        .map(|m| format!("M{m}"))
        .chain((1..=4).map(|q| format!("Q{q}")))
    {
        let filter = dynamic(&period);
        sheets.push((period, filter));
    }
    check_calculated("t4130-pivot-date-filters", &sheets);
}

#[test]
fn relative_dates_follow_the_day_given() {
    // Saturday 2026-11-07, the day the probe was filled.
    let today = 46_333.0;
    let covers = |name: &str, serial: f64| DynamicDate::from_name(name).covers(serial, Some(today));
    assert_eq!(
        DynamicDate::from_name("lastWeek"),
        DynamicDate::Relative(RelativeUnit::Week, -1)
    );
    assert_eq!(
        DynamicDate::from_name("nextQuarter"),
        DynamicDate::Relative(RelativeUnit::Quarter, 1)
    );
    assert!(covers("today", today + 0.75) && !covers("today", today + 1.0));
    assert!(covers("yesterday", today - 1.0) && covers("tomorrow", today + 1.0));
    // The week of Sunday 2026-11-01 to Saturday 2026-11-07.
    assert!(covers("thisWeek", today - 6.0) && !covers("thisWeek", today - 7.0));
    assert!(covers("lastWeek", today - 7.0) && covers("nextWeek", today + 1.0));
    assert!(covers("thisMonth", today - 6.0) && covers("lastMonth", today - 7.0));
    assert!(covers("nextMonth", today + 24.0) && !covers("nextMonth", today + 23.0));
    assert!(covers("thisQuarter", today - 37.0) && covers("lastQuarter", today - 38.0));
    assert!(covers("thisYear", today + 54.0) && covers("nextYear", today + 55.0));
    assert!(covers("lastYear", today - 311.0) && covers("yearToDate", today - 310.0));
    assert!(!covers("yearToDate", today + 1.0));
    assert_eq!(DynamicDate::from_name("Q3"), DynamicDate::Quarter(3));
    assert_eq!(DynamicDate::from_name("M13"), DynamicDate::Other);
    // With no day given a relative period keeps every item.
    assert!(DynamicDate::from_name("today").covers(1.0, None));
}

#[test]
fn filter_types_name_their_class() {
    for (name, kind) in [
        ("captionNotBetween", PivotFilterKind::Caption),
        ("valueEqual", PivotFilterKind::Value),
        ("count", Count),
        ("percent", Percent),
        ("sum", Sum),
        ("dateOlderThanOrEqual", PivotFilterKind::Date),
        ("M12", PivotFilterKind::Date),
        ("Q4", PivotFilterKind::Date),
        ("yearToDate", PivotFilterKind::Date),
        ("unknown", PivotFilterKind::Unknown),
    ] {
        assert_eq!(PivotFilterKind::from_type_name(name), kind, "{name}");
    }
}

#[test]
fn filters_charge_their_work() {
    let case = read("t4164-pivot-filter-totals", "case");
    let (cache, mut table) = probe_pivot(&case, "ITop");
    table.filters = vec![
        top10(1, Count, true, "1"),
        caption(1, &[(NotEqual, "A")], false),
    ];
    let records: Vec<u32> = (0..cache.record_count).collect();
    let mut work = MAX_PIVOT_WORK;
    let kept = apply(
        &table,
        &cache,
        records.clone(),
        FilterContext::default(),
        &mut work,
    )
    .unwrap();
    // East C 6 and West B 9 (A removed first, then the top one per region).
    assert_eq!(kept.records.len(), 2);
    let spent = MAX_PIVOT_WORK - work;
    assert!(spent > 0);
    // With less, there is no result.
    let mut work = spent - 1;
    assert!(matches!(
        apply(&table, &cache, records, FilterContext::default(), &mut work),
        Err(PivotComputeError::Limit)
    ));
}

#[test]
fn a_filter_on_a_field_off_the_axes_or_a_missing_measure_applies_nothing() {
    let case = read("t4164-pivot-filter-totals", "case");
    let (cache, plain) = probe_pivot(&case, "ITop");
    let mut work = MAX_PIVOT_WORK;
    let expected = compute(&plain, &cache, &mut work).unwrap();
    let mut table = plain.clone();
    let mut missing = top10(1, Count, true, "1");
    missing.measure = 7;
    // Field 2 (Sales) is on no axis.
    table.filters = vec![caption(2, &[(Equal, "nothing")], false), missing];
    let mut work = MAX_PIVOT_WORK;
    assert_eq!(compute(&table, &cache, &mut work).unwrap(), expected);
}

#[test]
fn a_huge_values_list_or_pattern_meets_the_budget_before_the_work() {
    let case = read("t4130-pivot-label-filters", "case");
    let (cache, mut table) = probe_pivot(&case, "CEq");
    let records: Vec<u32> = (0..cache.record_count).collect();
    let huge_list = FilterCriteria::Values {
        values: vec!["x".repeat(1 << 16); 1 << 8],
        blank: false,
        dates: Vec::new(),
    };
    let pattern = "*a".repeat(1 << 20);
    for filter in [
        filter(1, PivotFilterKind::Caption, huge_list),
        caption(1, &[(Equal, pattern.as_str())], false),
    ] {
        table.filters = vec![filter];
        // Enough for the plain pivot, far too little for folding 16 MB or matching the pattern
        // against every label.
        let mut work = 10_000;
        assert!(matches!(
            apply(
                &table,
                &cache,
                records.clone(),
                FilterContext::default(),
                &mut work
            ),
            Err(PivotComputeError::Limit)
        ));
        assert_eq!(work, 0);
        // With the full budget the filter runs, and keeps nothing.
        let mut work = MAX_PIVOT_WORK;
        let kept = apply(
            &table,
            &cache,
            records.clone(),
            FilterContext::default(),
            &mut work,
        );
        assert!(kept.unwrap().records.is_empty());
    }
}
