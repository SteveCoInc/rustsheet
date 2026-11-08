// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used, reason = "tests")]

//! Shared strings with leading or trailing whitespace are written with `xml:space="preserve"`
//! (T-0497; Excel writes it on such runs, e.g. `calc_tests/DATE_AND_TIME/DAYS_DAYS360.xlsx`).

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_to_xlsx;
use rustsheet_xlsx::import::load_from_xlsx;
use std::io::Read;

const TEXTS: [&str; 4] = [" a", "a ", "\n", "a b"];

#[test]
fn whitespace_edged_strings_are_preserved_on_export_and_round_trip() {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    for (i, text) in TEXTS.iter().enumerate() {
        model
            .update_cell_with_text(0, i as i32 + 1, 1, text)
            .unwrap();
    }
    let path = "temp_shared_string_whitespace.xlsx";
    save_to_xlsx(&model, path).unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(std::fs::File::open(path).unwrap())
        .unwrap()
        .by_name("xl/sharedStrings.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let reloaded = load_from_xlsx(path, "en", "UTC", "en").unwrap();
    std::fs::remove_file(path).unwrap();
    // The exporter escapes a newline as a character reference.
    for text in [" a", "a ", "&#xA;"] {
        assert!(
            xml.contains(&format!(r#"<t xml:space="preserve">{text}</t>"#)),
            "{text} in {xml}"
        );
    }
    assert!(xml.contains("<t>a b</t>"), "{xml}");
    for (i, text) in TEXTS.iter().enumerate() {
        assert_eq!(
            reloaded
                .get_formatted_cell_value(0, i as i32 + 1, 1)
                .unwrap(),
            *text
        );
    }
}
