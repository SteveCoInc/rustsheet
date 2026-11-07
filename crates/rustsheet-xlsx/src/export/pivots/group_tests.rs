#![allow(clippy::unwrap_used, reason = "tests")]
//! Grouped cache fields export with their `<fieldGroup>` and come back equal (T-4734): a numeric
//! range group, a date group of one level, nested date levels (Months in place, Quarters and
//! Years added) and a manual group, each with GETPIVOTDATA on its group items answering the same
//! after the round trip. The probes' grouped pivots (t4128 G, t4170, t4228, t4236, t4249, ...)
//! round-trip through the same export in rustsheet-eval's `pivot_definition::export_tests`.

use rustsheet_engine::pivot::{
    group_by_dates, group_by_discrete, group_by_numbers, keep_ungrouped, AddedGroup, Axis,
    AxisField, CacheBuilder, CacheField, CacheItem, CacheSource, CellArea, DataField, DateGroupBy,
    DateGroups, PivotCache, PivotField, PivotTable, RecordValue, MAX_RECORD_VALUES,
};
use rustsheet_engine::types::DateSystem;
use rustsheet_engine::Model;

use super::tests::{export, part, round_trip, without_lines};

/// 2019-01-01 and 2020-12-31, the date groups' bounds.
const START: f64 = 43466.0;
const END: f64 = 44196.0;
const AMOUNT: usize = 0;
const DATE: usize = 1;
const PRODUCT: usize = 2;
const S: u32 = 3;

/// Sheet1!A1:D7: Amount, Date, Product and S (distinct powers of two, so a sum names its
/// records). The dates: 2019-01-10, 2019-02-20, 2019-04-05, 2019-12-03, 2020-01-10 and 2017-09-23
/// (before startDate).
const RECORDS: [(f64, f64, &str, f64); 6] = [
    (5.0, 43475.0, "A", 1.0),
    (12.0, 43516.0, "B", 2.0),
    (18.0, 43560.0, "C", 4.0),
    (25.0, 43802.0, "A", 8.0),
    (31.0, 43840.0, "D", 16.0),
    (-1.0, 43001.0, "E", 32.0),
];

fn source_cache() -> PivotCache {
    let mut builder = CacheBuilder::new(
        Some(CacheSource {
            sheet_name: "Sheet1".into(),
            range: "A1:D7".into(),
        }),
        MAX_RECORD_VALUES,
    );
    for name in ["Amount", "Date", "Product", "S"] {
        builder.add_field(name.into()).unwrap();
    }
    for (amount, date, product, s) in RECORDS {
        let values = [
            CacheItem::Number(amount),
            CacheItem::Date(date),
            CacheItem::Text(product.into()),
            CacheItem::Number(s),
        ];
        builder
            .push_record(values.into_iter().map(RecordValue::Inline).collect())
            .unwrap();
    }
    let mut cache = builder.finish();
    cache.refresh_on_load = true;
    cache
}

fn texts(labels: &[&str]) -> Vec<CacheItem> {
    labels
        .iter()
        .map(|l| CacheItem::Text((*l).into()))
        .collect()
}

fn month_labels() -> Vec<CacheItem> {
    texts(&[
        "<1/1/2019",
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec",
        ">12/31/2020",
    ])
}

/// Field `field` of `cache` grouped in place by months.
fn group_by_months(cache: &mut PivotCache, field: usize) {
    let groups = DateGroups::new(DateGroupBy::Months, START, END, 14).unwrap();
    let base = cache.fields[field].clone();
    let grouped = group_by_dates(&base, groups, month_labels()).unwrap();
    cache.fields[field] = keep_ungrouped(base, grouped);
}

/// A level Excel adds over the date field (`Quarters`, `Years`), from the base's own values.
fn date_level(cache: &PivotCache, name: &str, by: DateGroupBy, labels: &[&str]) -> CacheField {
    let date = &cache.fields[DATE];
    let own = date.ungrouped.as_ref().unwrap();
    let base = CacheField {
        name: date.name.clone(),
        items: own.items.clone(),
        records: own.records.clone(),
        ..CacheField::default()
    };
    let groups = DateGroups::new(by, START, END, labels.len()).unwrap();
    let mut level = group_by_dates(&base, groups, texts(labels)).unwrap();
    level.name = name.into();
    level.added_group = Some(AddedGroup {
        base: 1,
        discrete: None,
    });
    level
}

/// A refreshOnLoad pivot of `cache` at Sheet1!F1:G12 with `rows` on the row axis and the sum of
/// S, and the GETPIVOTDATA `formulas` in column J.
fn model_of(cache: PivotCache, rows: &[u32], formulas: &[&str]) -> Model {
    let mut model = Model::new_empty("groups", "en", "UTC", "en").unwrap();
    let sheet_id = model.workbook.worksheets[0].sheet_id;
    for (column, name) in (1..).zip(["Amount", "Date", "Product", "S"]) {
        model.set_user_input(0, 1, column, name.into()).unwrap();
    }
    for (row, (amount, date, product, s)) in (2..).zip(RECORDS) {
        let values = [
            amount.to_string(),
            date.to_string(),
            product.into(),
            s.to_string(),
        ];
        for (column, value) in (1..).zip(values) {
            model.set_user_input(0, row, column, value).unwrap();
        }
    }
    let mut pivot = PivotTable::new(
        "Grouped".into(),
        sheet_id,
        CellArea {
            top: 0,
            left: 5,
            bottom: 11,
            right: 6,
        },
        0,
    );
    pivot.fields = (0u32..)
        .zip(&cache.fields)
        .map(|(i, f)| PivotField::new(rows.contains(&i).then_some(Axis::Row), f.items.len()))
        .collect();
    pivot.row_fields = rows.iter().copied().map(AxisField::Field).collect();
    pivot.data_fields = vec![DataField {
        caption: "Sum of S".into(),
        field: S,
        ..DataField::default()
    }];
    model.workbook.pivot_caches.push(cache);
    model.workbook.pivot_tables.push(pivot);
    for (row, formula) in (1..).zip(formulas) {
        model.set_user_input(0, row, 10, (*formula).into()).unwrap();
    }
    model
}

/// Exports and imports `model`: the cache comes back equal, the pivot reaches a fixed point, and
/// each formula of column J answers the same. Returns the answers and the cache definition part.
fn round_trips(mut model: Model, formulas: usize) -> (Vec<String>, String) {
    model.evaluate();
    let definition = part(&export(&model), "xl/pivotCache/pivotCacheDefinition1.xml");
    let mut back = round_trip(&model);
    assert_eq!(back.workbook.pivot_caches, model.workbook.pivot_caches);
    back.evaluate();
    let answers: Vec<String> = (1..=i32::try_from(formulas).unwrap())
        .map(|row| model.get_formatted_cell_value(0, row, 10).unwrap())
        .collect();
    for (row, answer) in (1..).zip(&answers) {
        assert_eq!(
            &back.get_formatted_cell_value(0, row, 10).unwrap(),
            answer,
            "J{row}"
        );
    }
    // The API pivot gains its `<items>` on the first trip (`an_api_pivot_round_trips_...`);
    // from then on cache and pivot are a fixed point.
    let again = round_trip(&back);
    assert_eq!(again.workbook.pivot_caches, back.workbook.pivot_caches);
    assert_eq!(
        without_lines(&again.workbook.pivot_tables),
        without_lines(&back.workbook.pivot_tables)
    );
    (answers, definition)
}

#[test]
fn a_numeric_range_group_round_trips() {
    let mut cache = source_cache();
    let base = cache.fields[AMOUNT].clone();
    let grouped = group_by_numbers(&base, [0.0, 29.0, 10.0], &[]).unwrap();
    cache.fields[AMOUNT] = keep_ungrouped(base, grouped);
    assert_eq!(
        cache.fields[AMOUNT].items,
        texts(&["<0", "0-9", "10-19", "20-29", ">30"])
    );
    let formulas = [
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Amount","10-19")"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Amount",10)"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Amount",">30")"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Amount","<0")"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1)"#,
    ];
    let model = model_of(cache, &[0], &formulas);
    let (answers, definition) = round_trips(model, formulas.len());
    assert_eq!(answers, ["6", "6", "16", "32", "63"]);
    // The field's own numbers are its shared items, its groups the `<fieldGroup>`.
    assert!(
        definition.contains(
            r#"<cacheField name="Amount" numFmtId="0"><sharedItems containsSemiMixedTypes="0" containsString="0" containsNumber="1" containsInteger="1" minValue="-1" maxValue="31" count="6"><n v="5"/>"#
        ),
        "{definition}"
    );
    assert!(
        definition.contains(
            r#"<fieldGroup base="0"><rangePr autoStart="0" autoEnd="0" startNum="0" endNum="29" groupInterval="10"/><groupItems count="5"><s v="&lt;0"/><s v="0-9"/><s v="10-19"/><s v="20-29"/><s v="&gt;30"/></groupItems></fieldGroup></cacheField>"#
        ),
        "{definition}"
    );
}

#[test]
fn a_date_group_of_one_level_round_trips() {
    let mut cache = source_cache();
    group_by_months(&mut cache, DATE);
    let formulas = [
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Date","Jan")"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Date",4)"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Date","<1/1/2019")"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Date","Dec")"#,
    ];
    let model = model_of(cache, &[1], &formulas);
    let (answers, definition) = round_trips(model, formulas.len());
    assert_eq!(answers, ["17", "4", "32", "8"]);
    assert!(
        definition.contains(r#"<d v="2019-01-10T00:00:00"/><d v="2019-02-20T00:00:00"/>"#),
        "{definition}"
    );
    assert!(
        definition.contains(
            r#"<fieldGroup base="1"><rangePr autoStart="0" autoEnd="0" groupBy="months" startDate="2019-01-01T00:00:00" endDate="2020-12-31T00:00:00"/><groupItems count="14"><s v="&lt;1/1/2019"/><s v="Jan"/>"#
        ),
        "{definition}"
    );
}

#[test]
fn nested_date_levels_round_trip() {
    let mut cache = source_cache();
    group_by_months(&mut cache, DATE);
    let quarters = date_level(
        &cache,
        "Quarters",
        DateGroupBy::Quarters,
        &["<1/1/2019", "Qtr1", "Qtr2", "Qtr3", "Qtr4", ">12/31/2020"],
    );
    let years = date_level(
        &cache,
        "Years",
        DateGroupBy::Years,
        &["<1/1/2019", "2019", "2020", ">12/31/2020"],
    );
    cache.fields.extend([quarters, years]);
    let formulas = [
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Years","2019")"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Years",2020)"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Years","<1/1/2019")"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1)"#,
    ];
    let model = model_of(cache.clone(), &[5], &formulas);
    let (answers, definition) = round_trips(model, formulas.len());
    assert_eq!(answers, ["15", "16", "32", "63"]);
    // The date field names its last added level; the levels have no values of their own.
    assert!(
        definition.contains(
            r#"<fieldGroup par="5" base="1"><rangePr autoStart="0" autoEnd="0" groupBy="months""#
        ),
        "{definition}"
    );
    assert!(
        definition.contains(
            r#"<cacheField name="Quarters" numFmtId="0" databaseField="0"><fieldGroup base="1"><rangePr autoStart="0" autoEnd="0" groupBy="quarters" startDate="2019-01-01T00:00:00" endDate="2020-12-31T00:00:00"/><groupItems count="6">"#
        ),
        "{definition}"
    );
    assert!(definition.contains(r#"<cacheFields count="6">"#));
    let mut plain = model_of(cache.clone(), &[5], &[]);
    plain.evaluate();
    let records = part(&export(&plain), "xl/pivotCache/pivotCacheRecords1.xml");
    // Four database values per record: the date as its own value, not its month.
    assert!(
        records
            .contains(r#"<r><x v="0"/><x v="0"/><x v="0"/><x v="0"/></r><r><x v="1"/><x v="1"/>"#),
        "{records}"
    );
    // All three levels on the rows answer the same too.
    let model = model_of(cache, &[5, 4, 1], &formulas);
    round_trips(model, formulas.len());
}

#[test]
fn a_manual_group_round_trips() {
    let mut cache = source_cache();
    // Product's items are A, B, C, D, E: A and B in G1, C and D in G2, E ungrouped.
    let labels = texts(&["E", "G1", "G2"]);
    let map = vec![1, 1, 2, 2, 0];
    let mut added = group_by_discrete(&cache.fields[PRODUCT], &map, labels).unwrap();
    added.name = "Product2".into();
    added.added_group = Some(AddedGroup {
        base: 2,
        discrete: Some(map),
    });
    cache.fields.push(added);
    let formulas = [
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Product2","G1")"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Product2","G2")"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Product2","E")"#,
        r#"=GETPIVOTDATA("Sum of S",$F$1,"Product","A")"#,
    ];
    let model = model_of(cache.clone(), &[4], &formulas);
    let (answers, definition) = round_trips(model, formulas.len());
    assert_eq!(&answers[..3], ["11", "20", "32"]);
    assert!(
        definition.contains(
            r#"<s v="E"/></sharedItems><fieldGroup par="4"/></cacheField><cacheField name="S""#
        ),
        "{definition}"
    );
    assert!(
        definition.contains(
            r#"<cacheField name="Product2" numFmtId="0" databaseField="0"><fieldGroup base="2"><discretePr count="5"><x v="1"/><x v="1"/><x v="2"/><x v="2"/><x v="0"/></discretePr><groupItems count="3"><s v="E"/><s v="G1"/><s v="G2"/></groupItems></fieldGroup></cacheField>"#
        ),
        "{definition}"
    );
    // The group field and its base on the rows answer the same too.
    round_trips(model_of(cache, &[4, 2], &formulas), formulas.len());
}

/// Groups of shapes the export does not write, or that would not read back as they are, drop
/// the cache with its pivots (and a log line), never write it half.
#[test]
fn groups_the_import_would_not_read_back_drop_the_cache() {
    use super::cache::check;
    let check = |cache: &PivotCache| check(cache, DateSystem::Excel1900, MAX_RECORD_VALUES);
    let mut grouped = source_cache();
    group_by_months(&mut grouped, DATE);
    assert!(check(&grouped).is_ok());

    // Group items that are not what the base values group to.
    let mut wrong = grouped.clone();
    wrong.fields[DATE].records[0] = 2;
    assert!(check(&wrong).is_err());
    // Own values beside no grouping.
    let mut stray = source_cache();
    stray.fields[AMOUNT].ungrouped = grouped.fields[DATE].ungrouped.clone();
    assert!(check(&stray).is_err());
    // A manual group beside a range grouping, and a numeric level Excel never adds.
    let mut beside = grouped.clone();
    let mut added =
        group_by_discrete(&source_cache().fields[DATE], &[0; 6], texts(&["x"])).unwrap();
    added.added_group = Some(AddedGroup {
        base: 1,
        discrete: Some(vec![0; 6]),
    });
    beside.fields.push(added);
    assert!(check(&beside).is_err());
    let mut numeric = source_cache();
    let mut level = group_by_numbers(&numeric.fields[AMOUNT], [0.0, 29.0, 10.0], &[]).unwrap();
    level.added_group = Some(AddedGroup {
        base: 0,
        discrete: None,
    });
    numeric.fields.push(level);
    assert!(check(&numeric).is_err());
    // An added level that keeps values of its own, which would not come back.
    let mut kept = grouped.clone();
    let mut level = date_level(
        &grouped,
        "Years",
        DateGroupBy::Years,
        &["<", "2019", "2020", ">"],
    );
    level.ungrouped = grouped.fields[DATE].ungrouped.clone();
    kept.fields.push(level);
    assert!(check(&kept).is_err());
    // A database field after an added one.
    let mut order = grouped.clone();
    let level = date_level(
        &grouped,
        "Years",
        DateGroupBy::Years,
        &["<", "2019", "2020", ">"],
    );
    order.fields.insert(2, level);
    assert!(check(&order).is_err());
    // Calculated fields beside a grouping: the import skips such a cache (T-4174).
    let mut calculated = grouped.clone();
    calculated.fields.push(CacheField {
        name: "Twice".into(),
        formula: Some("S*2".into()),
        ..CacheField::default()
    });
    assert!(check(&calculated).is_err());
    // A date group in a 1904 workbook, which the import refuses.
    assert!(super::cache::check(&grouped, DateSystem::Excel1904, MAX_RECORD_VALUES).is_err());

    let mut model = model_of(wrong, &[1], &[]);
    model.evaluate();
    let back = round_trip(&model);
    assert!(back.workbook.pivot_caches.is_empty());
    assert!(back.workbook.pivot_tables.is_empty());
}
