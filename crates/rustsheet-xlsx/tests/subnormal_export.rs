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
    // Unprobed (T-4355): a negative kept subnormal keeps its value in the file.
    model
        .set_user_input(0, 5, 1, "=-EXP(-745)".to_string())
        .unwrap();
    model.evaluate();
    let path = "temp_subnormal_export.xlsx";
    save_to_xlsx(&model, path).unwrap();
    assert_eq!(cached(path, "A1"), "0");
    // Only the value is pinned: the spelling of a number in the file is out of scope (T-2211).
    assert_eq!(
        cached(path, "A2").parse::<f64>().unwrap(),
        3.307_553_003_638_408e-308
    );
    assert_eq!(cached(path, "A3"), "0");
    let mut reloaded = load_from_xlsx(path, "en", "UTC", "en").unwrap();
    std::fs::remove_file(path).unwrap();
    reloaded.evaluate();
    // The formulas are saved with their text, so recalculation gives 2^-1074 again.
    assert_eq!(
        reloaded.get_cell_value_by_index(0, 1, 1).unwrap(),
        CellValue::Number(f64::from_bits(1))
    );
    assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "FALSE");
}
