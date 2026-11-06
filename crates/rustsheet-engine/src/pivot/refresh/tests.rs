#![allow(clippy::unwrap_used, reason = "tests")]
//! Tests of [`refresh`] against the cache-level expectations of the item and field-name probes:
//! each source is the probe's data sheet, each old cache what the case builder saved, and each
//! expectation what Excel saved after its refresh on open (the sidecars' `cache_fields` and
//! `rowItems`).

use super::*;
use crate::expressions::token::Error;
use crate::test::util::new_empty_model;

/// A value typed into a source cell: the input text and an optional number format.
type Input<'a> = (&'a str, Option<&'a str>);

/// A model whose Sheet1 holds `columns` from A1 down, evaluated.
fn source(columns: &[&[Input]]) -> Model {
    let mut model = new_empty_model();
    for (column, cells) in (1..).zip(columns) {
        for (row, (input, format)) in (1..).zip(cells.iter()) {
            if input.is_empty() {
                continue;
            }
            model
                .set_user_input(0, row, column, (*input).to_string())
                .unwrap();
            if let Some(format) = format {
                let mut style = model.get_style_for_cell(0, row, column).unwrap();
                style.num_fmt = (*format).to_string();
                model.set_cell_style(0, row, column, &style).unwrap();
            }
        }
    }
    model.evaluate();
    model
}

fn plain<'a>(inputs: &[&'a str]) -> Vec<Input<'a>> {
    inputs.iter().map(|i| (*i, None)).collect()
}

/// A cache over Sheet1!`range` whose fields list `fields` (name, items).
fn cache(range: &str, fields: Vec<(&str, Vec<CacheItem>)>) -> PivotCache {
    PivotCache {
        source: Some(CacheSource {
            sheet_name: "Sheet1".to_string(),
            range: range.to_string(),
        }),
        fields: fields
            .into_iter()
            .map(|(name, items)| CacheField {
                name: name.to_string(),
                items,
                ..CacheField::default()
            })
            .collect(),
        record_count: 0,
    }
}

fn text(s: &str) -> CacheItem {
    CacheItem::Text(s.to_string())
}

fn num(n: f64) -> CacheItem {
    CacheItem::Number(n)
}

/// The K field's items and records after refreshing `old` over `model`.
fn refreshed_k(model: &Model, old: &PivotCache) -> (Vec<CacheItem>, Vec<u32>) {
    let fresh = refresh(old, model, MAX_RECORD_VALUES).unwrap();
    let k = fresh.fields.into_iter().next().unwrap();
    (k.items, k.records)
}

const V4: [&str; 5] = ["V", "1", "1", "1", "1"];

/// t4134 I1: east, East and EAST are one item, shown `east`; the builder listed `east`.
#[test]
fn text_items_ignore_case() {
    let model = source(&[&plain(&["K", "east", "East", "EAST", "west"]), &plain(&V4)]);
    let old = cache(
        "A1:B5",
        vec![("K", vec![text("east"), text("west")]), ("V", vec![])],
    );
    let (items, records) = refreshed_k(&model, &old);
    assert_eq!(items, [text("east"), text("west")]);
    assert_eq!(records, [0, 0, 0, 1]);
}

/// t4582 J1: the cached `EAST` slot is reused and relabelled with the source's casing, and the
/// cached `<m/>` slot stays, holding no record (saved rowItems x = 1, 2).
#[test]
fn a_cached_text_item_keeps_its_slot_and_takes_the_source_casing() {
    let model = source(&[
        &plain(&["K", "=\"east\"", "EAST", "East", "west"]),
        &plain(&V4),
    ]);
    let old = cache(
        "A1:B5",
        vec![
            ("K", vec![CacheItem::Missing, text("EAST"), text("west")]),
            ("V", vec![]),
        ],
    );
    let (items, records) = refreshed_k(&model, &old);
    assert_eq!(items, [CacheItem::Missing, text("east"), text("west")]);
    assert_eq!(records, [1, 1, 1, 2]);
}

/// t4134 I2: the listed order of mixed types is kept (saved rowItems x = 0..=7).
#[test]
fn listed_items_keep_their_order_across_types() {
    let model = source(&[
        &plain(&["K", "2", "b", "TRUE", "#N/A", "", "1", "a", "FALSE"]),
        &plain(&["V", "1", "1", "1", "1", "1", "1", "1", "1"]),
    ]);
    let listed = vec![
        text("a"),
        CacheItem::Boolean(false),
        num(1.0),
        text("b"),
        CacheItem::Boolean(true),
        num(2.0),
        CacheItem::Error(Error::NA),
        CacheItem::Missing,
    ];
    let old = cache("A1:B9", vec![("K", listed.clone()), ("V", vec![])]);
    let (items, records) = refreshed_k(&model, &old);
    assert_eq!(items, listed);
    assert_eq!(records, [5, 3, 4, 6, 7, 2, 0, 1]);
}

/// t4134 I3 and t4582 J2: items only the refresh sees follow the listed ones in source order
/// (Zed, Alpha; Alpha, Zed, Beta), and the listed `<m/>` keeps index 1 with no record (saved
/// rowItems x = 0, 2, 3 and 0, 2, 3, 4).
#[test]
fn new_items_are_appended_in_source_order() {
    let model = source(&[
        &plain(&["K", "M", "=\"Zed\"", "=\"Alpha\"", "M"]),
        &plain(&V4),
    ]);
    let old = cache(
        "A1:B5",
        vec![("K", vec![text("M"), CacheItem::Missing]), ("V", vec![])],
    );
    let (items, records) = refreshed_k(&model, &old);
    assert_eq!(
        items,
        [text("M"), CacheItem::Missing, text("Zed"), text("Alpha")]
    );
    assert_eq!(records, [0, 2, 3, 0]);

    let model = source(&[
        &plain(&["K", "M", "=\"Alpha\"", "=\"Zed\"", "=\"Beta\"", "M"]),
        &plain(&["V", "1", "1", "1", "1", "1"]),
    ]);
    let old = cache(
        "A1:B6",
        vec![("K", vec![text("M"), CacheItem::Missing]), ("V", vec![])],
    );
    let (items, records) = refreshed_k(&model, &old);
    assert_eq!(
        items,
        [
            text("M"),
            CacheItem::Missing,
            text("Alpha"),
            text("Zed"),
            text("Beta")
        ]
    );
    assert_eq!(records, [0, 2, 3, 4, 0]);
}

/// t4134 I4 and I5: numbers compare exactly, and the text `1` is not the number 1.
#[test]
fn numbers_are_exact_and_text_is_never_a_number() {
    let model = source(&[
        &plain(&["K", "0.3", "=0.1+0.2", "0.3"]),
        &plain(&["V", "1", "1", "1"]),
    ]);
    let old = cache(
        "A1:B4",
        vec![
            ("K", vec![num(0.3), num(0.300_000_000_000_000_04)]),
            ("V", vec![]),
        ],
    );
    let (items, records) = refreshed_k(&model, &old);
    assert_eq!(items, [num(0.3), num(0.300_000_000_000_000_04)]);
    assert_eq!(records, [0, 1, 0]);

    let model = source(&[
        &plain(&["K", "1", "'1", "1"]),
        &plain(&["V", "1", "1", "1"]),
    ]);
    let old = cache(
        "A1:B4",
        vec![("K", vec![num(1.0), text("1")]), ("V", vec![])],
    );
    let (items, records) = refreshed_k(&model, &old);
    assert_eq!(items, [num(1.0), text("1")]);
    assert_eq!(records, [0, 1, 0]);
}

/// t4582 J3 and J4: date-formatted values never match the cached number items; they are new
/// items, 45000.5 its own (saved rowItems x = 3, 4, 5).
#[test]
fn date_values_do_not_match_cached_numbers() {
    for format in ["m/d/yyyy", "m/d/yyyy h:mm"] {
        let k: Vec<Input> = [("K", None)]
            .into_iter()
            .chain(["45000", "45000.5", "45000", "45001"].map(|v| (v, Some(format))))
            .collect();
        let model = source(&[&k, &plain(&V4)]);
        let old = cache(
            "A1:B5",
            vec![
                ("K", vec![num(45000.0), num(45000.5), num(45001.0)]),
                ("V", vec![]),
            ],
        );
        let (items, records) = refreshed_k(&model, &old);
        let dates = [45000.0, 45000.5, 45001.0].map(CacheItem::Date);
        assert_eq!(items[..3], [num(45000.0), num(45000.5), num(45001.0)]);
        assert_eq!(items[3..], dates, "{format}");
        assert_eq!(records, [3, 4, 3, 5], "{format}");
    }
}

/// t4585: repeated headers are numbered from 2 per name ignoring case, keeping their casing,
/// and the empty-text header in the 4th column of an A-start source is `Field4`.
#[test]
fn repeated_and_blank_headers_are_renamed() {
    let model = source(&[
        &plain(&["Region", "East", "West"]),
        &plain(&["Sales", "10", "20"]),
        &plain(&["=\"Sales\"", "1", "3"]),
        &plain(&["=\"\"", "2", "4"]),
        &plain(&["=\"region\"", "a", "b"]),
        &plain(&["=\"Sales\"&\"\"", "5", "6"]),
    ]);
    // The builder saved the formula headers under their formula text.
    let old = cache(
        "A1:F3",
        [
            "Region",
            "Sales",
            "=\"Sales\"",
            "=\"\"",
            "=\"region\"",
            "=\"Sales\"&\"\"",
        ]
        .map(|n| (n, vec![]))
        .to_vec(),
    );
    let fresh = refresh(&old, &model, MAX_RECORD_VALUES).unwrap();
    let names: Vec<&str> = fresh.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(
        names,
        ["Region", "Sales", "Sales2", "Field4", "region2", "Sales3"]
    );
    assert_eq!(fresh.record_count, 2);
    assert_eq!(fresh.fields[0].items, [text("East"), text("West")]);
    assert_eq!(fresh.fields[1].items, [num(10.0), num(20.0)]);
    assert_eq!(fresh.fields[1].records, [0, 1]);
}

/// probes/t4627-pivot-field-name-collisions: a taken name takes the first free `<header><k>`
/// from 2 (`P_SufA`, `P_SufB`, `P_Const`), whitespace is a name (`P_Space`), `ß` and `SS` are
/// one name (`P_Fold`), and a blank is `Field<n>` by source position, named after the text
/// headers (`P_FldA`, `P_FldB`, `P_Empty`).
#[test]
fn field_names_avoid_every_taken_name() {
    fn all<'a>(headers: &[&'a str]) -> Vec<Option<&'a str>> {
        headers.iter().map(|h| Some(*h)).collect()
    }
    let names = |headers: &[Option<&str>]| {
        field_names(headers.iter().map(|h| h.map_or(CacheItem::Missing, text))).unwrap()
    };
    assert_eq!(
        names(&all(&["Region", "Sales", "Sales2", "Sales"])),
        ["Region", "Sales", "Sales2", "Sales3"]
    );
    assert_eq!(
        names(&all(&["Region", "Sales", "Sales", "Sales2"])),
        ["Region", "Sales", "Sales2", "Sales22"]
    );
    assert_eq!(
        names(&all(&["Region", "Sales", "Sales", "SALES"])),
        ["Region", "Sales", "Sales2", "SALES3"]
    );
    assert_eq!(names(&all(&[" ", " "])), [" ", " 2"]);
    assert_eq!(
        names(&all(&["é", "É", "ß", "SS", "ss"])),
        ["é", "É2", "ß", "SS2", "ss3"]
    );
    assert_eq!(
        names(&all(&["Region", "Sales", "Field4", ""])),
        ["Region", "Sales", "Field4", "Field42"]
    );
    assert_eq!(
        names(&all(&["Region", "Sales", "X", "", "Field4"])),
        ["Region", "Sales", "X", "Field42", "Field4"]
    );
    assert_eq!(
        names(&[Some("Region"), Some("Sales"), Some("X"), None, Some("Y")]),
        ["Region", "Sales", "X", "Field4", "Y"]
    );
}

/// Untrusted input (directive 5): [`MAX_FIELDS`] equal headers, or two repeated headers whose
/// suffixed names collide, are named in linear time, each search resuming where its base's left
/// off.
#[test]
fn many_equal_headers_are_named_in_linear_time() {
    let n = MAX_FIELDS;
    let names = field_names((0..n).map(|_| text("Sales"))).unwrap();
    assert_eq!(names.len(), n);
    assert_eq!(names[1], "Sales2");
    assert_eq!(names[n - 1], format!("Sales{n}"));
    // `A2` repeated meets the `A2<k>` names the repeated `A` took first: `A` took `A2` to
    // `A8192`, so `A22` to `A2999` (k = 2 to 999) are taken and the first `A2` repeat is `A21000`.
    let half = n / 2;
    let mixed = field_names((0..n).map(|i| text(if i < half { "A" } else { "A2" }))).unwrap();
    let distinct: HashSet<String> = mixed.iter().map(|m| name_key(m)).collect();
    assert_eq!(distinct.len(), n);
    assert_eq!(mixed[half], "A21000");
    assert_eq!(mixed[half + 1], "A21001");
}

#[test]
fn a_non_text_header_or_a_missing_source_is_refused() {
    let model = source(&[&plain(&["K", "a"]), &plain(&["7", "1"])]);
    let old = cache("A1:B2", vec![]);
    assert_eq!(
        refresh(&old, &model, MAX_RECORD_VALUES),
        Err(PivotCacheError::Header { column: 1 })
    );
    for (sheet, range) in [("Nope", "A1:B2"), ("Sheet1", "Table1"), ("Sheet1", "A1")] {
        let old = PivotCache {
            source: Some(CacheSource {
                sheet_name: sheet.to_string(),
                range: range.to_string(),
            }),
            ..PivotCache::default()
        };
        assert_eq!(
            refresh(&old, &model, MAX_RECORD_VALUES),
            Err(PivotCacheError::Source),
            "{range}"
        );
    }
    assert_eq!(
        refresh(&PivotCache::default(), &model, MAX_RECORD_VALUES),
        Err(PivotCacheError::Source)
    );
}

/// The limits are checked on the source's size before any cell is read.
#[test]
fn the_record_value_budget_is_checked_first() {
    let model = source(&[&plain(&["K", "a", "b"]), &plain(&["V", "1", "2"])]);
    let old = cache("A1:B3", vec![]);
    assert_eq!(
        refresh(&old, &model, 3),
        Err(PivotCacheError::TooManyRecordValues)
    );
    assert!(refresh(&old, &model, 4).is_ok());
}

/// A field with a saved numeric grouping is grouped again over the new values, its labels
/// kept; over a blank the grouping is dropped. The expectation is `group_by_numbers` over the
/// read values (the importer's grouping step), not Excel's: no probe refreshes a grouped field's
/// cache yet (PV3).
#[test]
fn a_grouped_field_is_grouped_again() {
    let model = source(&[&plain(&["N", "1", "4", "7"]), &plain(&["V", "1", "1", "1"])]);
    let labels: Vec<CacheItem> = ["<0", "0-2", "3-5", "6-8", ">9"].map(text).to_vec();
    let base = cache("A1:B4", vec![("N", vec![num(1.0)]), ("V", vec![])]);
    let mut grouped = base.clone();
    grouped.fields[0] = group_by_numbers(&base.fields[0], [0.0, 9.0, 3.0], &labels).unwrap();
    let fresh = refresh(&grouped, &model, MAX_RECORD_VALUES).unwrap();
    assert_eq!(fresh.fields[0].items, grouped.fields[0].items);
    assert!(fresh.fields[0].number_groups.is_some());
    // The values 1, 4 and 7 fall in the groups their base field falls in.
    let read = CacheField {
        name: "N".to_string(),
        items: vec![num(1.0), num(4.0), num(7.0)],
        records: vec![0, 1, 2],
        ..CacheField::default()
    };
    let expected = group_by_numbers(&read, [0.0, 9.0, 3.0], &grouped.fields[0].items).unwrap();
    assert_eq!(fresh.fields[0], expected);
    assert_eq!(fresh.fields[0].records.len(), 3);

    let model = source(&[&plain(&["N", "1", "", "7"]), &plain(&["V", "1", "1", "1"])]);
    let fresh = refresh(&grouped, &model, MAX_RECORD_VALUES).unwrap();
    assert_eq!(
        fresh.fields[0].items,
        [num(1.0), CacheItem::Missing, num(7.0)]
    );
    assert!(fresh.fields[0].number_groups.is_none());
}

#[test]
fn shared_items_flags_follow_the_items() {
    let field = |items: Vec<CacheItem>| CacheField {
        items,
        ..CacheField::default()
    };
    let numbers = SharedItemsFlags::of(&field(vec![num(2.0), num(-1.0)]));
    assert!(!numbers.contains_semi_mixed_types && !numbers.contains_string);
    assert!(numbers.contains_number && numbers.contains_integer);
    assert_eq!(numbers.number_range, Some((-1.0, 2.0)));
    let with_blank = SharedItemsFlags::of(&field(vec![num(0.5), CacheItem::Missing]));
    assert!(with_blank.contains_blank && with_blank.contains_semi_mixed_types);
    assert!(!with_blank.contains_integer && !with_blank.contains_mixed_types);
    let mixed = SharedItemsFlags::of(&field(vec![CacheItem::Boolean(true), num(1.0)]));
    assert!(mixed.contains_string && mixed.contains_mixed_types);
    let errors = SharedItemsFlags::of(&field(vec![CacheItem::Error(Error::NA)]));
    assert!(errors.contains_string && !errors.contains_number);
}

/// A field with a saved date grouping is grouped again by `group_by_dates` over the dates read,
/// its labels and grouping kept. As for numbers, the expectation is the grouping step's, pending
/// a refresh probe of grouped fields (PV3).
#[test]
fn a_date_grouped_field_is_grouped_again() {
    let model = source(&[
        &[
            ("D", None),
            ("1/1/2019", None),
            ("5/5/2019", None),
            ("2/1/2019", None),
        ],
        &plain(&["V", "1", "1", "1"]),
    ]);
    let months: Vec<CacheItem> = ["<1/1/2019", "Jan", "Feb", "Mar", "Apr", "May", "Jun"]
        .into_iter()
        .chain(["Jul", "Aug", "Sep", "Oct", "Nov", "Dec", ">12/31/2019"])
        .map(text)
        .collect();
    let groups = DateGroups::new(DateGroupBy::Months, 43466.0, 43830.0, months.len()).unwrap();
    let base = cache("A1:B4", vec![("D", vec![num(43466.0)]), ("V", vec![])]);
    let mut grouped = base.clone();
    grouped.fields[0] = group_by_dates(&base.fields[0], groups, months.clone()).unwrap();
    let fresh = refresh(&grouped, &model, MAX_RECORD_VALUES).unwrap();
    let read = CacheField {
        name: "D".to_string(),
        items: [43466.0, 43590.0, 43497.0].map(CacheItem::Date).to_vec(),
        records: vec![0, 1, 2],
        ..CacheField::default()
    };
    assert_eq!(
        fresh.fields[0],
        group_by_dates(&read, groups, months).unwrap()
    );
    assert_eq!(fresh.fields[0].date_groups, Some(groups));
    assert_eq!(fresh.fields[0].records, [1, 5, 2]);
}
