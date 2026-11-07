#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::test::util::new_empty_model;

/// Fuzz finding (nightly `formula_eval`, run 38344232074, T-1467): FIXED with about 1.2e8
/// decimals built a string of over 100 MB. Decimals of 128 or more are `#VALUE!`
/// (docs/excel-semantics/text.md, T-1281).
#[test]
fn fixed_and_dollar_with_huge_decimals_are_value_errors() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.rename_sheet("Sheet2", "Data").expect("rename");
    model.set_a1("Data!E9", "1.5");
    model.set_a1("Data!D3", "a");
    model.set_a1("Data!C2", "b");
    model.set_a1(
        "A1",
        "=ENCODEURL(CONCATENATE(FIXED(Data!E9,123456789.123,TRUE),(Data!D3&Data!$C2)))",
    );
    model.set_a1("A2", "=DOLLAR(0,123456789)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
}
