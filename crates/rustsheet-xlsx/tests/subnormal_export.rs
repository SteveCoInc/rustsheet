#![allow(clippy::unwrap_used, reason = "tests")]

use rustsheet_engine::{cell::CellValue, Model};
use rustsheet_xlsx::export::save_to_xlsx;
use rustsheet_xlsx::import::load_from_xlsx;
use std::io::Read;

/// The `<v>` text of the cell `r` in sheet 1.
fn cached(path: &str, r: &str) -> String {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut xml = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let cell = xml
        .split("<c ")
        .find(|c| c.starts_with(&format!("r=\"{r}\"")))
        .unwrap();
    cell.split("<v>")
        .nth(1)
        .unwrap()
        .split("</v>")
        .next()
        .unwrap()
        .to_string()
}

/// T-2211, decision 0089: Excel for the web saves a kept subnormal formula result as `<v>0</v>`
/// (probes/subnormal-saved-as-zero) and a normal one in full; the in-memory value is untouched.
#[test]
fn kept_subnormal_is_saved_as_zero() {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    for (row, input) in [(1, "=EXP(-745)"), (2, "=EXP(-708)"), (3, "=A1")] {
        model.set_user_input(0, row, 1, input.to_string()).unwrap();
    }
    model
        .set_user_input(0, 4, 1, "=(EXP(-745))=0".to_string())
        .unwrap();
    // T-4355 (probes/subnormal-saved-negative-and-array A1): a negative kept subnormal is saved as 0
    // too. `0-x` keeps the sign; `-EXP(-745)` would be +2^-1074 (numbers.md "Unary minus of a tiny subnormal").
    model
        .set_user_input(0, 6, 1, "=0-EXP(-745)".to_string())
        .unwrap();
    // The array path is saved as 0 as well (probe cells A4, A5, T-4355).
    model
        .set_user_input(0, 5, 1, "=EXP({-745,-708})".to_string())
        .unwrap();
    model.evaluate();
    let path = "temp_subnormal_export.xlsx";
    save_to_xlsx(&model, path).unwrap();
    // Read everything before asserting, so a failure does not leave the file behind.
    let saved: Vec<String> = ["A1", "A2", "A3", "A5", "B5", "A6"]
        .iter()
        .map(|r| cached(path, r))
        .collect();
    let reloaded = load_from_xlsx(path, "en", "UTC", "en");
    std::fs::remove_file(path).unwrap();
    let mut reloaded = reloaded.unwrap();
    let full = |s: &str| s.parse::<f64>().unwrap();
    assert_eq!(saved[0], "0");
    // Only the value is pinned: the spelling of a number in the file is out of scope (T-2211).
    assert_eq!(full(&saved[1]), 3.307_553_003_638_408e-308);
    assert_eq!(saved[2], "0");
    assert_eq!(saved[3], "0");
    assert_eq!(full(&saved[4]), 3.307_553_003_638_408e-308);
    assert_eq!(saved[5], "0");
    reloaded.evaluate();
    // The formulas are saved with their text, so recalculation gives 2^-1074 again.
    assert_eq!(
        reloaded.get_cell_value_by_index(0, 1, 1).unwrap(),
        CellValue::Number(f64::from_bits(1))
    );
    assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "FALSE");
}
