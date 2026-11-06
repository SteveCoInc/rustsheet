#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! The probe harness the pivot stage tests share: a probe's case file (its source cells and the
//! case builder's pivot definition) read back into the cache and definition Excel refreshed, and its
//! oracle sidecar (Excel's saved `pivots`: cells, `rowItems`/`colItems` and `location`), so every
//! expectation comes from the committed probe files, never from the engine.

use serde_json::Value as Json;

use crate::expressions::token::get_error_by_english_name;
use crate::expressions::utils::number_to_column;
use crate::pivot::{
    read_source, Aggregate, Axis, AxisField, CacheBuilder, CacheItem, CacheSource, CellArea,
    DataField, PageField, PivotCache, PivotField, PivotItem, PivotTable, RecordValue, SourceArea,
    SourceCells, MAX_RECORD_VALUES,
};

pub(crate) fn read(stem: &str, kind: &str) -> Json {
    let path = format!(
        "{}/../../eval/suites/probes/cases/{stem}.{kind}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

pub(crate) fn sheet<'a>(case: &'a Json, name: &str) -> &'a Json {
    case["sheets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == name)
        .unwrap_or_else(|| panic!("no sheet {name}"))
}

/// `(column, row)`, both 0-based, of an A1 address.
pub(crate) fn position(address: &str) -> (u32, u32) {
    let split = address.find(|c: char| c.is_ascii_digit()).unwrap();
    let (letters, digits) = address.split_at(split);
    let column = letters
        .bytes()
        .fold(0, |acc, b| acc * 26 + u32::from(b - b'A' + 1));
    (column - 1, digits.parse::<u32>().unwrap() - 1)
}

pub(crate) fn address(column: u32, row: u32) -> String {
    let letters = number_to_column(i32::try_from(column + 1).unwrap()).unwrap();
    format!("{letters}{}", row + 1)
}

/// A case cell's value as a cache holds it.
pub(crate) fn cache_item(value: Option<&Json>) -> CacheItem {
    match value {
        None => CacheItem::Missing,
        Some(Json::String(text)) => {
            assert!(!text.starts_with('='), "a formula in a source: {text}");
            CacheItem::Text(text.clone())
        }
        Some(Json::Number(n)) => CacheItem::Number(n.as_f64().unwrap()),
        Some(Json::Bool(b)) => CacheItem::Boolean(*b),
        // A typed text cell: text that would read as a number (probes/t4134-pivot-items D5!A3).
        Some(typed) if typed["t"] == "s" => CacheItem::Text(typed["v"].as_str().unwrap().into()),
        Some(error) => {
            assert_eq!(error["t"], "e");
            CacheItem::Error(get_error_by_english_name(error["v"].as_str().unwrap()).unwrap())
        }
    }
}

/// A case sheet's cells as a pivot source.
struct CaseCells<'a> {
    cells: &'a Json,
    area: SourceArea,
}

impl SourceCells for CaseCells<'_> {
    fn resolve(&self, _: &CacheSource) -> Option<SourceArea> {
        Some(self.area)
    }

    fn value(&self, _: u32, row: u32, column: u32) -> CacheItem {
        cache_item(self.cells.get(address(column, row)))
    }
}

/// The cache Excel's refresh builds from `source` (`Data!A1:H5`), read by the engine's own
/// refresh (`read_source`, with Excel's item identity): items in first-seen order, as the case
/// builder lists them (`pivot_definition.py` `_Field.items`). A formula in the source panics:
/// the case file holds no evaluated value for it.
pub(crate) fn source_cache(case: &Json, source: &str) -> PivotCache {
    let (name, range) = source.split_once('!').unwrap();
    let (from, to) = range.split_once(':').unwrap();
    let ((left, top), (right, bottom)) = (position(from), position(to));
    let cells = CaseCells {
        cells: &sheet(case, name)["cells"],
        area: SourceArea {
            sheet: 0,
            area: CellArea {
                top,
                left,
                bottom,
                right,
            },
        },
    };
    let prior = PivotCache {
        source: Some(CacheSource {
            sheet_name: name.to_string(),
            range: range.to_string(),
        }),
        ..PivotCache::default()
    };
    read_source(&prior, &cells, MAX_RECORD_VALUES)
        .unwrap()
        .finish()
}

pub(crate) fn aggregate_named(name: &str) -> Aggregate {
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
pub(crate) fn subtotal_named(name: &str) -> Aggregate {
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
pub(crate) fn item_of(cache: &PivotCache, field: u32, value: &Json) -> u32 {
    let wanted = cache_item(Some(value));
    let items = &cache.fields[field as usize].items;
    u32::try_from(items.iter().position(|i| *i == wanted).unwrap()).unwrap()
}

/// The pivot the case builder writes for the definition of sheet `name`'s pivot.
pub(crate) fn probe_pivot(case: &Json, name: &str) -> (PivotCache, PivotTable) {
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
    // The anchor: the builder's `location` is the body's top-left cell (page fields above it).
    let (left, top) = position(spec["location"].as_str().unwrap());
    let area = CellArea {
        top,
        left,
        bottom: top,
        right: left,
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
        let spec = &definition["fields"][&cache_field.name];
        if axis.is_some() {
            // The builder lists the field's `order` first, then the rest in first-seen order.
            let first: Vec<u32> = spec["order"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|value| item_of(&cache, index, value))
                .collect();
            let rest = (0..u32::try_from(cache_field.items.len()).unwrap())
                .filter(|item| !first.contains(item));
            pivot_field.items = first
                .iter()
                .copied()
                .chain(rest)
                .map(PivotItem::data)
                .collect();
        }
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
    // The form flags as the builder writes them: `outline` and `outlineData` on unless `table`
    // sets them (`_TABLE_DEFAULTS`), the rest at the schema's default unless set.
    let flag = |value: &Json, default: bool| value.as_bool().unwrap_or(default);
    let o = &mut table.options;
    o.compact = flag(&options["compact"], o.compact);
    o.outline = flag(&options["outline"], true);
    o.compact_data = flag(&options["compactData"], o.compact_data);
    o.outline_data = flag(&options["outlineData"], true);
    o.row_grand_totals = flag(&options["rowGrandTotals"], o.row_grand_totals);
    o.column_grand_totals = flag(&options["colGrandTotals"], o.column_grand_totals);
    o.page_over_then_down = flag(&options["pageOverThenDown"], o.page_over_then_down);
    if let Some(wrap) = options["pageWrap"].as_u64() {
        o.page_wrap = u32::try_from(wrap).unwrap();
    }
    for (cache_field, pivot_field) in cache.fields.iter().zip(&mut table.fields) {
        let attrs = &definition["fields"][&cache_field.name]["attrs"];
        pivot_field.compact = flag(&attrs["compact"], pivot_field.compact);
        pivot_field.outline = flag(&attrs["outline"], pivot_field.outline);
        pivot_field.subtotal_top = flag(&attrs["subtotalTop"], pivot_field.subtotal_top);
    }
    (cache, table)
}

/// A diagonal pivot: record `i` is row item `i` and column item `i`, Sum of its index.
pub(crate) fn diagonal(n: u32, data: bool) -> (PivotCache, PivotTable) {
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    for name in ["R", "C", "V"] {
        builder.add_field(name.to_string()).unwrap();
    }
    for i in 0..n {
        builder
            .push_record(vec![
                RecordValue::Inline(CacheItem::Number(f64::from(i))),
                RecordValue::Inline(CacheItem::Number(f64::from(i))),
                RecordValue::Inline(CacheItem::Number(f64::from(i))),
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
    let mut table = PivotTable::new("D".to_string(), 1, area, 0);
    let len = |f: usize| cache.fields[f].items.len();
    table.fields = vec![
        PivotField::new(Some(Axis::Row), len(0)),
        PivotField::new(Some(Axis::Column), len(1)),
        PivotField::new(None, len(2)),
    ];
    table.row_fields = vec![AxisField::Field(0)];
    table.column_fields = vec![AxisField::Field(1)];
    if data {
        table.data_fields = vec![DataField {
            field: 2,
            ..DataField::default()
        }];
    }
    (cache, table)
}
