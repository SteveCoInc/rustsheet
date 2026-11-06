#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! Every value cell of the oracle-rendered aggregate probes, against the result tree: the
//! source, the definition and the expected cells all come from the probes' case and sidecar
//! files (probes/t4135-pivot-aggregates, t4589-pivot-total-order, t4614-pivot-total-order-2,
//! t4137-pivot-captions), read through the sidecar's own line encoding (`rowItems`/`colItems`).

use serde_json::Value as Json;

use super::*;
use crate::expressions::token::get_error_by_english_name;
use crate::pivot::{
    CacheBuilder, CellArea, DataField, ItemType, PageField, PivotItem, RecordValue,
    MAX_RECORD_VALUES,
};

fn read(stem: &str, kind: &str) -> Json {
    let path = format!(
        "{}/../../eval/suites/probes/cases/{stem}.{kind}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

fn sheet<'a>(case: &'a Json, name: &str) -> &'a Json {
    case["sheets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == name)
        .unwrap_or_else(|| panic!("no sheet {name}"))
}

/// `(column, row)`, both 0-based, of an A1 address.
fn position(address: &str) -> (u32, u32) {
    let split = address.find(|c: char| c.is_ascii_digit()).unwrap();
    let (letters, digits) = address.split_at(split);
    let column = letters
        .bytes()
        .fold(0, |acc, b| acc * 26 + u32::from(b - b'A' + 1));
    (column - 1, digits.parse::<u32>().unwrap() - 1)
}

fn address(column: u32, row: u32) -> String {
    let mut letters = String::new();
    let mut n = column + 1;
    while n > 0 {
        let rem = (n - 1) % 26;
        letters.insert(0, char::from(b'A' + u8::try_from(rem).unwrap()));
        n = (n - 1) / 26;
    }
    format!("{letters}{}", row + 1)
}

fn cache_item(value: Option<&Json>) -> CacheItem {
    match value {
        None => CacheItem::Missing,
        Some(Json::String(text)) => {
            assert!(!text.starts_with('='), "a formula in a source: {text}");
            CacheItem::Text(text.clone())
        }
        Some(Json::Number(n)) => CacheItem::Number(n.as_f64().unwrap()),
        Some(Json::Bool(b)) => CacheItem::Boolean(*b),
        Some(error) => {
            assert_eq!(error["t"], "e");
            CacheItem::Error(get_error_by_english_name(error["v"].as_str().unwrap()).unwrap())
        }
    }
}

/// The cache Excel's refresh builds from `source` (`Data!A1:H5`): items in first-seen order, as
/// the case builder lists them (`pivot_definition.py` `_Field.items`).
fn source_cache(case: &Json, source: &str) -> PivotCache {
    let (name, range) = source.split_once('!').unwrap();
    let (from, to) = range.split_once(':').unwrap();
    let ((left, top), (right, bottom)) = (position(from), position(to));
    let cells = &sheet(case, name)["cells"];
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    for column in left..=right {
        let header = cells[address(column, top)].as_str().unwrap();
        builder.add_field(header.to_string()).unwrap();
    }
    for row in top + 1..=bottom {
        let values = (left..=right)
            .map(|column| RecordValue::Inline(cache_item(cells.get(address(column, row)))))
            .collect();
        builder.push_record(values).unwrap();
    }
    builder.finish()
}

fn aggregate_named(name: &str) -> Aggregate {
    match name {
        "sum" => Aggregate::Sum,
        "count" => Aggregate::Count,
        "average" => Aggregate::Average,
        "max" => Aggregate::Max,
        "min" => Aggregate::Min,
        "product" => Aggregate::Product,
        "countNums" => Aggregate::CountNums,
        "stdDev" => Aggregate::StdDev,
        "stdDevp" => Aggregate::StdDevP,
        "var" => Aggregate::Var,
        "varp" => Aggregate::VarP,
        _ => panic!("unknown aggregate {name}"),
    }
}

/// A custom subtotal line's function by its `item@t`, as the importer reads it.
fn subtotal_named(name: &str) -> Aggregate {
    match name {
        "countA" => Aggregate::Count,
        "count" => Aggregate::CountNums,
        "avg" => Aggregate::Average,
        "stdDevP" => Aggregate::StdDevP,
        "varP" => Aggregate::VarP,
        other => aggregate_named(other),
    }
}

/// A field's cache item equal to `value`.
fn item_of(cache: &PivotCache, field: u32, value: &Json) -> u32 {
    let wanted = cache_item(Some(value));
    let items = &cache.fields[field as usize].items;
    u32::try_from(items.iter().position(|i| *i == wanted).unwrap()).unwrap()
}

/// The pivot the case builder writes for the definition of sheet `name`'s pivot.
fn probe_pivot(case: &Json, name: &str) -> (PivotCache, PivotTable) {
    let spec = &sheet(case, name)["pivot_tables"][0];
    let cache = source_cache(case, spec["source"].as_str().unwrap());
    let definition = &spec["definition"];
    let field = |name: &str| {
        u32::try_from(cache.fields.iter().position(|f| f.name == name).unwrap()).unwrap()
    };
    let names = |key: &str| -> Vec<String> {
        definition[key].as_array().map_or_else(Vec::new, |a| {
            a.iter()
                .map(|v| {
                    v.as_str()
                        .unwrap_or_else(|| v["field"].as_str().unwrap())
                        .to_string()
                })
                .collect()
        })
    };
    let axis = |names: &[String]| -> Vec<AxisField> {
        names
            .iter()
            .map(|n| match n.as_str() {
                "Values" => AxisField::Values,
                n => AxisField::Field(field(n)),
            })
            .collect()
    };
    let area = CellArea {
        top: 0,
        left: 0,
        bottom: 0,
        right: 0,
    };
    let mut table = PivotTable::new(name.to_string(), 1, area, 0);
    table.row_fields = axis(&names("rows"));
    table.column_fields = axis(&names("columns"));
    let data = definition["data"].as_array().unwrap();
    if data.len() > 1
        && !table.row_fields.contains(&AxisField::Values)
        && !table.column_fields.contains(&AxisField::Values)
    {
        table.column_fields.push(AxisField::Values);
    }
    let pages = names("pages");
    for (index, cache_field) in (0u32..).zip(&cache.fields) {
        let on = |list: &[AxisField]| list.contains(&AxisField::Field(index));
        let axis = if on(&table.row_fields) {
            Some(Axis::Row)
        } else if on(&table.column_fields) {
            Some(Axis::Column)
        } else if pages.contains(&cache_field.name) {
            Some(Axis::Page)
        } else {
            None
        };
        let mut pivot_field = PivotField::new(axis, cache_field.items.len());
        if axis.is_some() {
            pivot_field.items = (0..u32::try_from(cache_field.items.len()).unwrap())
                .map(PivotItem::data)
                .collect();
        }
        let spec = &definition["fields"][&cache_field.name];
        for hidden in spec["hidden"].as_array().into_iter().flatten() {
            pivot_field.hide(item_of(&cache, index, hidden));
        }
        if let Some(subtotals) = spec["subtotals"].as_array() {
            pivot_field.default_subtotal = false;
            pivot_field.custom_subtotals = subtotals
                .iter()
                .map(|s| subtotal_named(s.as_str().unwrap()))
                .collect();
        }
        table.fields.push(pivot_field);
    }
    for page in definition["pages"].as_array().into_iter().flatten() {
        let index = field(page["field"].as_str().unwrap());
        table.page_fields.push(PageField {
            field: index,
            item: page.get("item").map(|item| item_of(&cache, index, item)),
            hierarchy: Some(-1),
        });
    }
    table.data_fields = data
        .iter()
        .map(|d| DataField {
            caption: d["name"].as_str().unwrap_or_default().to_string(),
            field: field(d["field"].as_str().unwrap()),
            function: aggregate_named(d["subtotal"].as_str().unwrap_or("sum")),
            ..DataField::default()
        })
        .collect();
    let options = &definition["table"];
    table.options.show_error = options["showError"].as_bool().unwrap_or(false);
    table.options.error_caption = options["errorCaption"].as_str().map(str::to_string);
    table.options.missing_caption = options["missingCaption"].as_str().map(str::to_string);
    (cache, table)
}

/// One line of a sidecar's `rowItems`/`colItems`: the cache items of its real fields, its data
/// field and, for a custom subtotal line, its function.
struct Line {
    items: Vec<u32>,
    data: usize,
    function: Option<Aggregate>,
}

/// Decodes `items` (each `x` after the `r` items it repeats from the line before) over `axis`.
fn lines(items: &Json, axis: &[AxisField]) -> Vec<Line> {
    let mut previous: Vec<u32> = Vec::new();
    let mut out = Vec::new();
    for line in items.as_array().unwrap() {
        let repeat = usize::try_from(line["r"].as_u64().unwrap_or(0)).unwrap();
        let mut full: Vec<u32> = previous.iter().copied().take(repeat).collect();
        full.extend(
            line["x"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| u32::try_from(x.as_u64().unwrap()).unwrap()),
        );
        previous.clone_from(&full);
        let kind = line["t"].as_str().unwrap_or("data");
        let items = if kind == "grand" {
            Vec::new()
        } else {
            full.iter()
                .zip(axis)
                .filter(|(_, f)| **f != AxisField::Values)
                .map(|(x, _)| *x)
                .collect()
        };
        out.push(Line {
            items,
            data: usize::try_from(line["i"].as_u64().unwrap_or(0)).unwrap(),
            function: match kind {
                "data" | "grand" | "default" => None,
                custom => Some(subtotal_named(custom)),
            },
        });
    }
    out
}

/// Checks every value cell of the sheet `name`'s pivot of probe `stem`; returns the number of
/// cells checked.
fn check(stem: &str, name: &str) -> usize {
    let (case, oracle) = (read(stem, "case"), read(stem, "oracle"));
    let (cache, table) = probe_pivot(&case, name);
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let pivot = oracle["pivots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["sheet"] == name)
        .unwrap();
    let location = pivot["location"].as_str().unwrap();
    let (left, top) = position(location.split(':').next().unwrap());
    let first_row = u32::try_from(pivot["first_data_row"].as_u64().unwrap()).unwrap();
    let first_column = u32::try_from(pivot["first_data_col"].as_u64().unwrap()).unwrap();
    let values_on_rows = table.row_fields.contains(&AxisField::Values);
    let row_lines = lines(&pivot["row_items"], &table.row_fields);
    let column_lines = lines(&pivot["col_items"], &table.column_fields);
    let error_caption = table.options.error_caption.clone();
    let missing_caption = table.options.missing_caption.clone();
    let mut wrong = Vec::new();
    let mut checked = 0;
    for (i, row_line) in (0u32..).zip(&row_lines) {
        for (j, column_line) in (0u32..).zip(&column_lines) {
            let cell = address(left + first_column + j, top + first_row + i);
            let data = if values_on_rows {
                row_line.data
            } else {
                column_line.data
            };
            let function = row_line
                .function
                .or(column_line.function)
                .unwrap_or(table.data_fields[data].function);
            let actual = match (
                result.rows().find(&row_line.items),
                result.columns().find(&column_line.items),
            ) {
                (Some(row), Some(column)) => result.value_as(row, column, data, function),
                _ => None,
            };
            let expected = &pivot["cells"][&cell];
            let text = expected["v"].as_str();
            let ok = match (expected["t"].as_str(), &actual) {
                (None | Some("empty"), None) => true,
                (Some("n"), Some(Ok(n))) => expected["v"].as_f64() == Some(*n),
                (Some("e"), Some(Err(e))) => text == Some(e.to_string().as_str()),
                (Some("s"), Some(Err(_))) => {
                    table.options.show_error && text == error_caption.as_deref()
                }
                (Some("s"), None) => text == missing_caption.as_deref(),
                _ => false,
            };
            if !ok {
                wrong.push(format!(
                    "{name}!{cell}: Excel {expected}, result {actual:?}"
                ));
            }
            checked += 1;
        }
    }
    assert!(wrong.is_empty(), "{stem}:\n{}", wrong.join("\n"));
    checked
}

#[test]
fn every_t4135_aggregate_cell_is_excels() {
    // Eleven aggregates over numbers, text, blanks, booleans, an error and decimals, with Kind
    // on columns (A_*), and the eleven over the error field under errorCaption (EC).
    let sheets = [
        "A_sum",
        "A_count",
        "A_average",
        "A_max",
        "A_min",
        "A_product",
        "A_countNums",
        "A_stdDev",
        "A_stdDevp",
        "A_var",
        "A_varp",
        "EC",
    ];
    let checked: usize = sheets
        .iter()
        .map(|s| check("t4135-pivot-aggregates", s))
        .sum();
    assert_eq!(checked, 11 * 3 * 18 + 3 * 11);
}

#[test]
fn every_t4589_total_order_cell_is_excels() {
    for sheet in ["S1", "E1", "E2", "E3", "E4"] {
        check("t4589-pivot-total-order", sheet);
    }
}

#[test]
fn every_t4614_total_order_cell_is_excels() {
    for sheet in ["S2", "S3", "P2", "C1", "C2", "C3", "C4"] {
        check("t4614-pivot-total-order-2", sheet);
    }
}

#[test]
fn every_t4137_caption_pivot_value_is_excels() {
    // K4: a page selection and hidden items of a page field; K5: custom Sum and Count subtotals;
    // K3: a #DIV/0! under errorCaption; K2: an empty intersection under missingCaption.
    for sheet in ["K1", "K2", "K3", "K4", "K5", "K6"] {
        check("t4137-pivot-captions", sheet);
    }
}

/// One text field `K` over `keys`, and a number field `V` over `values`, Sum of V by K on rows.
fn small(keys: &[&str], values: &[f64]) -> (PivotCache, PivotTable) {
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    builder.add_field("K".to_string()).unwrap();
    builder.add_field("V".to_string()).unwrap();
    for (k, v) in keys.iter().zip(values) {
        builder
            .push_record(vec![
                RecordValue::Inline(CacheItem::Text((*k).to_string())),
                RecordValue::Inline(CacheItem::Number(*v)),
            ])
            .unwrap();
    }
    let cache = builder.finish();
    let area = CellArea {
        top: 0,
        left: 0,
        bottom: 0,
        right: 0,
    };
    let mut table = PivotTable::new("P".to_string(), 1, area, 0);
    let items = cache.fields[0].items.len();
    table.fields = vec![
        PivotField::new(Some(Axis::Row), items),
        PivotField::new(None, cache.fields[1].items.len()),
    ];
    table.row_fields = vec![AxisField::Field(0)];
    table.data_fields = vec![DataField {
        field: 1,
        ..DataField::default()
    }];
    (cache, table)
}

fn grand(table: &PivotTable, cache: &PivotCache) -> Option<Result<f64, Error>> {
    let mut work = MAX_PIVOT_WORK;
    compute(table, cache, &mut work).unwrap().value(0, 0, 0)
}

#[test]
fn a_page_item_that_names_no_data_item_reads_as_all() {
    let (cache, mut table) = small(&["a", "b"], &[1.0, 2.0]);
    table.fields[0].axis = Some(Axis::Page);
    table.row_fields.clear();
    table.fields[0].items = vec![
        PivotItem::data(1),
        PivotItem {
            kind: ItemType::Default,
            index: None,
            expanded: true,
            filtered: false,
        },
    ];
    let page = |item| PageField {
        field: 0,
        item,
        hierarchy: Some(-1),
    };
    // The pivot item 0 is the cache item 1 (`b`).
    table.page_fields = vec![page(Some(0))];
    assert_eq!(grand(&table, &cache), Some(Ok(2.0)));
    for malformed in [Some(1), Some(7), None] {
        table.page_fields = vec![page(malformed)];
        assert_eq!(grand(&table, &cache), Some(Ok(3.0)), "{malformed:?}");
    }
}

#[test]
fn hidden_items_filter_on_an_axis_and_not_off_every_axis() {
    let (cache, mut table) = small(&["a", "b"], &[1.0, 2.0]);
    table.fields[0].hide(0);
    assert_eq!(grand(&table, &cache), Some(Ok(2.0)));
    table.fields[0].axis = None;
    table.row_fields.clear();
    assert_eq!(grand(&table, &cache), Some(Ok(3.0)));
}

#[test]
fn the_listed_item_order_is_the_display_and_accumulation_order() {
    // t4614 S2: D1's records, Z first seen, Sum beside Average. Listed A first, the total
    // accumulates A then Z (29.599999999999998); in first-seen order Z then A, Excel's 29.6
    // (t4614 S2!B7).
    let keys = ["Z", "A", "Z", "A", "Z", "A"];
    let (cache, mut table) = small(&keys, &[6.5, 0.1, 2.3, 8.9, 2.4, 9.4]);
    table.data_fields.push(DataField {
        field: 1,
        function: Aggregate::Average,
        ..DataField::default()
    });
    assert_eq!(grand(&table, &cache), Some(Ok(29.6)));
    table.fields[0].items = vec![PivotItem::data(1), PivotItem::data(0)];
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let items: Vec<Option<u32>> = result.rows().nodes().iter().map(|n| n.item).collect();
    assert_eq!(items, [None, Some(1), Some(0)]);
    assert_eq!(result.value(0, 0, 0), Some(Ok(29.599_999_999_999_998)));
}

#[test]
fn an_empty_result_has_a_root_and_no_value() {
    let (cache, mut table) = small(&["a"], &[1.0]);
    table.fields[0].hide(0);
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    assert!(result.records().is_empty());
    assert_eq!(result.rows().nodes().len(), 1);
    assert_eq!(result.value(0, 0, 0), None);
}

#[test]
fn past_the_work_it_is_given_the_compute_fails_and_spends_it_all() {
    let (cache, table) = small(&["a", "b", "c"], &[1.0, 2.0, 3.0]);
    let mut work = 5;
    assert_eq!(
        compute(&table, &cache, &mut work),
        Err(PivotComputeError::Limit)
    );
    assert_eq!(work, 0);
}

#[test]
fn a_field_past_the_cache_is_malformed() {
    let (cache, mut table) = small(&["a"], &[1.0]);
    table.data_fields[0].field = 9;
    let mut work = MAX_PIVOT_WORK;
    assert_eq!(
        compute(&table, &cache, &mut work),
        Err(PivotComputeError::Malformed)
    );
}
