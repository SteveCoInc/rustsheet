#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::Model;

#[test]
fn a_volatile_field_selector_is_evaluated_once() {
    // Audit 2 item 4: the error-cell check evaluated the field's selector, and the driver then
    // evaluated it again, so `INDEX(D1:E1,RANDBETWEEN(1,2))` could be checked on D1 (#N/A) and
    // read from E1, or checked on E1 and read as D1's #N/A, which no single pick gives. Now the
    // pick is made once: D1 is the error-cell `#VALUE!`, E1 the Qty sum.
    let mut model = new_empty_model();
    model.set_a1("A1", "Qty");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("C1", "Qty");
    model.set_a1("D1", "=NA()");
    model.set_a1("E1", "Qty");
    model.set_a1("G1", "=DSUM(A1:A3,INDEX(D1:E1,RANDBETWEEN(1,2)),C1:C2)");
    for _ in 0..64 {
        model.evaluate();
        let text = model.get_text_a1("G1");
        assert!(matches!(text.as_str(), "#VALUE!" | "5"), "{text}");
    }
}

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DMIN()");
    model.set_a1("A2", "=DMIN(2)");
    model.set_a1("A3", "=DMIN(1, 2)");
    model.set_a1("A4", "=DMIN(1, 2, 3, 4)");

    model.set_a1("A5", "=DMAX()");
    model.set_a1("A6", "=DMAX(2)");
    model.set_a1("A7", "=DMAX(1, 2)");
    model.set_a1("A8", "=DMAX(1, 2, 3, 4)");

    model.set_a1("A9", "=DAVERAGE()");
    model.set_a1("A10", "=DAVERAGE(2)");
    model.set_a1("A11", "=DAVERAGE(1, 2)");
    model.set_a1("A12", "=DAVERAGE(1, 2, 3, 4)");

    model.set_a1("A13", "=DSUM()");
    model.set_a1("A14", "=DSUM(2)");
    model.set_a1("A15", "=DSUM(1, 2)");
    model.set_a1("A16", "=DSUM(1, 2, 3, 4)");

    model.set_a1("A17", "=DCOUNT()");
    model.set_a1("A18", "=DCOUNT(2)");
    model.set_a1("A19", "=DCOUNT(1, 2)");
    model.set_a1("A20", "=DCOUNT(1, 2, 3, 4)");

    model.set_a1("A21", "=DCOUNTA()");
    model.set_a1("A22", "=DCOUNTA(2)");
    model.set_a1("A23", "=DCOUNTA(1, 2)");
    model.set_a1("A24", "=DCOUNTA(1, 2, 3, 4)");

    model.set_a1("B1", "=DGET()");
    model.set_a1("B2", "=DGET(2)");
    model.set_a1("B3", "=DGET(1, 2)");
    model.set_a1("B4", "=DGET(1, 2, 3, 4)");

    model.set_a1("B5", "=DPRODUCT()");
    model.set_a1("B6", "=DPRODUCT(2)");
    model.set_a1("B7", "=DPRODUCT(1, 2)");
    model.set_a1("B8", "=DPRODUCT(1, 2, 3, 4)");

    model.set_a1("B9", "=DVAR()");
    model.set_a1("B10", "=DVAR(2)");
    model.set_a1("B11", "=DVAR(1, 2)");
    model.set_a1("B12", "=DVAR(1, 2, 3, 4)");

    model.set_a1("B13", "=DVARP()");
    model.set_a1("B14", "=DVARP(2)");
    model.set_a1("B15", "=DVARP(1, 2)");
    model.set_a1("B16", "=DVARP(1, 2, 3, 4)");

    model.set_a1("B17", "=DSTDEV()");
    model.set_a1("B18", "=DSTDEV(2)");
    model.set_a1("B19", "=DSTDEV(1, 2)");
    model.set_a1("B20", "=DSTDEV(1, 2, 3, 4)");

    model.set_a1("B21", "=DSTDEVP()");
    model.set_a1("B22", "=DSTDEVP(2)");
    model.set_a1("B23", "=DSTDEVP(1, 2)");
    model.set_a1("B24", "=DSTDEVP(1, 2, 3, 4)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A6"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A7"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A8"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A9"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A10"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A11"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A12"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A13"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A14"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A15"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A16"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A17"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A18"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A19"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A20"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A21"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A22"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A23"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A24"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B4"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B5"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B6"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B7"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B8"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B9"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B10"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B11"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B12"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B13"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B14"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B15"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B16"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B17"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B18"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B19"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B20"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B21"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B22"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B23"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B24"), *"#ERROR!");
}

#[test]
fn locale_iso_format() {
    // ISO format with YYYY-MM-DD format. Works with any locale.
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();

    // Create database
    model.set_a1("A1", "ID");
    model.set_a1("B1", "Date");
    model.set_a1("C1", "Amount");
    model.set_a1("A2", "1");
    model.set_a1("B2", "2026-01-15");
    model.set_a1("C2", "1200");
    model.set_a1("A3", "2");
    model.set_a1("B3", "2026-03-15");
    model.set_a1("C3", "900");
    model.set_a1("A4", "3");
    model.set_a1("B4", "2026-06-15");
    model.set_a1("C4", "2100");

    // Define criteria
    model.set_a1("A6", "Date");
    model.set_a1("A7", ">=2026-03-01");
    model.set_a1("B6", "Date");
    model.set_a1("B7", "2026-01-15"); // DGET needs an exact match

    // Test functions
    model.set_a1("A9", "=DMIN(A1:C4, C1, A6:A7)");
    model.set_a1("A10", "=DMAX(A1:C4, C1, A6:A7)");
    model.set_a1("A11", "=DAVERAGE(A1:C4, C1, A6:A7)");
    model.set_a1("A12", "=DSUM(A1:C4, C1, A6:A7)");
    model.set_a1("A13", "=DPRODUCT(A1:C4, C1, A6:A7)");
    model.set_a1("A14", "=DGET(A1:C4, C1, B6:B7)");
    model.set_a1("A15", "=DCOUNT(A1:C4, C1, A6:A7)");
    model.set_a1("A16", "=DCOUNTA(A1:C4, C1, A6:A7)");
    model.set_a1("A17", "=DVAR(A1:C4, C1, A6:A7)");
    model.set_a1("A18", "=DVARP(A1:C4, C1, A6:A7)");
    model.set_a1("A19", "=DSTDEV(A1:C4, C1, A6:A7)");
    model.set_a1("A20", "=DSTDEVP(A1:C4, C1, A6:A7)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A9"), *"900");
    assert_eq!(model.get_text_a1("A10"), *"2100");
    assert_eq!(model.get_text_a1("A11"), *"1500");
    assert_eq!(model.get_text_a1("A12"), *"3000");
    assert_eq!(model.get_text_a1("A13"), *"1890000");
    assert_eq!(model.get_text_a1("A14"), *"1200");
    assert_eq!(model.get_text_a1("A15"), *"2");
    assert_eq!(model.get_text_a1("A16"), *"2");
    assert_eq!(model.get_text_a1("A17"), *"720000");
    assert_eq!(model.get_text_a1("A18"), *"360000");
    assert_eq!(model.get_text_a1("A19"), *"848.5281374");
    assert_eq!(model.get_text_a1("A20"), *"600");
}

#[test]
fn locale_uk() {
    // en-GB locale with DD/MM/YYYY format
    let mut model = Model::new_empty("model", "en-GB", "UTC", "en").unwrap();

    // Create database
    model.set_a1("A1", "ID");
    model.set_a1("B1", "Date");
    model.set_a1("C1", "Amount");
    model.set_a1("A2", "1");
    model.set_a1("B2", "15/01/2026");
    model.set_a1("C2", "1200");
    model.set_a1("A3", "2");
    model.set_a1("B3", "15/03/2026");
    model.set_a1("C3", "900");
    model.set_a1("A4", "3");
    model.set_a1("B4", "15/06/2026");
    model.set_a1("C4", "2100");

    // Define criteria
    model.set_a1("A6", "Date");
    model.set_a1("A7", ">=01/03/2026");
    model.set_a1("B6", "Date");
    model.set_a1("B7", "15/01/2026"); // DGET needs an exact match

    // Test functions
    model.set_a1("A9", "=DMIN(A1:C4, C1, A6:A7)");
    model.set_a1("A10", "=DMAX(A1:C4, C1, A6:A7)");
    model.set_a1("A11", "=DAVERAGE(A1:C4, C1, A6:A7)");
    model.set_a1("A12", "=DSUM(A1:C4, C1, A6:A7)");
    model.set_a1("A13", "=DPRODUCT(A1:C4, C1, A6:A7)");
    model.set_a1("A14", "=DGET(A1:C4, C1, B6:B7)");
    model.set_a1("A15", "=DCOUNT(A1:C4, C1, A6:A7)");
    model.set_a1("A16", "=DCOUNTA(A1:C4, C1, A6:A7)");
    model.set_a1("A17", "=DVAR(A1:C4, C1, A6:A7)");
    model.set_a1("A18", "=DVARP(A1:C4, C1, A6:A7)");
    model.set_a1("A19", "=DSTDEV(A1:C4, C1, A6:A7)");
    model.set_a1("A20", "=DSTDEVP(A1:C4, C1, A6:A7)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A9"), *"900");
    assert_eq!(model.get_text_a1("A10"), *"2100");
    assert_eq!(model.get_text_a1("A11"), *"1500");
    assert_eq!(model.get_text_a1("A12"), *"3000");
    assert_eq!(model.get_text_a1("A13"), *"1890000");
    assert_eq!(model.get_text_a1("A14"), *"1200");
    assert_eq!(model.get_text_a1("A15"), *"2");
    assert_eq!(model.get_text_a1("A16"), *"2");
    assert_eq!(model.get_text_a1("A17"), *"720000");
    assert_eq!(model.get_text_a1("A18"), *"360000");
    assert_eq!(model.get_text_a1("A19"), *"848.5281374");
    assert_eq!(model.get_text_a1("A20"), *"600");
}

#[test]
fn locale_us() {
    // en-US locale with MM/DD/YY format
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();

    // Create database
    model.set_a1("A1", "ID");
    model.set_a1("B1", "Date");
    model.set_a1("C1", "Amount");
    model.set_a1("A2", "1");
    model.set_a1("B2", "1/15/26");
    model.set_a1("C2", "1200");
    model.set_a1("A3", "2");
    model.set_a1("B3", "3/15/26");
    model.set_a1("C3", "900");
    model.set_a1("A4", "3");
    model.set_a1("B4", "6/15/26");
    model.set_a1("C4", "2100");

    // Define criteria
    model.set_a1("A6", "Date");
    model.set_a1("A7", ">=3/1/26");
    model.set_a1("B6", "Date");
    model.set_a1("B7", "1/15/26"); // DGET needs an exact match

    // Test functions
    model.set_a1("A9", "=DMIN(A1:C4, C1, A6:A7)");
    model.set_a1("A10", "=DMAX(A1:C4, C1, A6:A7)");
    model.set_a1("A11", "=DAVERAGE(A1:C4, C1, A6:A7)");
    model.set_a1("A12", "=DSUM(A1:C4, C1, A6:A7)");
    model.set_a1("A13", "=DPRODUCT(A1:C4, C1, A6:A7)");
    model.set_a1("A14", "=DGET(A1:C4, C1, B6:B7)");
    model.set_a1("A15", "=DCOUNT(A1:C4, C1, A6:A7)");
    model.set_a1("A16", "=DCOUNTA(A1:C4, C1, A6:A7)");
    model.set_a1("A17", "=DVAR(A1:C4, C1, A6:A7)");
    model.set_a1("A18", "=DVARP(A1:C4, C1, A6:A7)");
    model.set_a1("A19", "=DSTDEV(A1:C4, C1, A6:A7)");
    model.set_a1("A20", "=DSTDEVP(A1:C4, C1, A6:A7)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A9"), *"900");
    assert_eq!(model.get_text_a1("A10"), *"2100");
    assert_eq!(model.get_text_a1("A11"), *"1500");
    assert_eq!(model.get_text_a1("A12"), *"3000");
    assert_eq!(model.get_text_a1("A13"), *"1890000");
    assert_eq!(model.get_text_a1("A14"), *"1200");
    assert_eq!(model.get_text_a1("A15"), *"2");
    assert_eq!(model.get_text_a1("A16"), *"2");
    assert_eq!(model.get_text_a1("A17"), *"720000");
    assert_eq!(model.get_text_a1("A18"), *"360000");
    assert_eq!(model.get_text_a1("A19"), *"848.5281374");
    assert_eq!(model.get_text_a1("A20"), *"600");
}

#[test]
fn locale_de() {
    // de-DE locale with D.M.YYYY format
    let mut model = Model::new_empty("model", "de", "UTC", "en").unwrap();

    // Create database
    model.set_a1("A1", "ID");
    model.set_a1("B1", "Date");
    model.set_a1("C1", "Amount");
    model.set_a1("A2", "1");
    model.set_a1("B2", "15.1.2026");
    model.set_a1("C2", "1200");
    model.set_a1("A3", "2");
    model.set_a1("B3", "15.3.2026");
    model.set_a1("C3", "900");
    model.set_a1("A4", "3");
    model.set_a1("B4", "15.6.2026");
    model.set_a1("C4", "2100");

    // Define criteria
    model.set_a1("A6", "Date");
    model.set_a1("A7", ">=1.3.2026");
    model.set_a1("B6", "Date");
    model.set_a1("B7", "15.1.2026"); // DGET needs an exact match

    // Test functions
    model.set_a1("A9", "=DMIN(A1:C4; C1; A6:A7)");
    model.set_a1("A10", "=DMAX(A1:C4; C1; A6:A7)");
    model.set_a1("A11", "=DAVERAGE(A1:C4; C1; A6:A7)");
    model.set_a1("A12", "=DSUM(A1:C4; C1; A6:A7)");
    model.set_a1("A13", "=DPRODUCT(A1:C4; C1; A6:A7)");
    model.set_a1("A14", "=DGET(A1:C4; C1; B6:B7)");
    model.set_a1("A15", "=DCOUNT(A1:C4; C1; A6:A7)");
    model.set_a1("A16", "=DCOUNTA(A1:C4; C1; A6:A7)");
    model.set_a1("A17", "=DVAR(A1:C4; C1; A6:A7)");
    model.set_a1("A18", "=DVARP(A1:C4; C1; A6:A7)");
    model.set_a1("A19", "=DSTDEV(A1:C4; C1; A6:A7)");
    model.set_a1("A20", "=DSTDEVP(A1:C4; C1; A6:A7)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A9"), *"900");
    assert_eq!(model.get_text_a1("A10"), *"2100");
    assert_eq!(model.get_text_a1("A11"), *"1500");
    assert_eq!(model.get_text_a1("A12"), *"3000");
    assert_eq!(model.get_text_a1("A13"), *"1890000");
    assert_eq!(model.get_text_a1("A14"), *"1200");
    assert_eq!(model.get_text_a1("A15"), *"2");
    assert_eq!(model.get_text_a1("A16"), *"2");
    assert_eq!(model.get_text_a1("A17"), *"720000");
    assert_eq!(model.get_text_a1("A18"), *"360000");
    assert_eq!(model.get_text_a1("A19"), *"848,5281374");
    assert_eq!(model.get_text_a1("A20"), *"600");
}

#[test]
fn locale_wrong_format() {
    // en-US locale with incorrect D.M.YYYY format
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();

    // Create database
    model.set_a1("A1", "ID");
    model.set_a1("B1", "Date");
    model.set_a1("C1", "Amount");
    model.set_a1("A2", "1");
    model.set_a1("B2", "10.1.2026");
    model.set_a1("C2", "1200");
    model.set_a1("A3", "2");
    model.set_a1("B3", "10.3.2026");
    model.set_a1("C3", "900");
    model.set_a1("A4", "3");
    model.set_a1("B4", "10.6.2026");
    model.set_a1("C4", "2100");

    // Define criteria
    model.set_a1("A6", "Date");
    model.set_a1("A7", ">=1.3.2026");
    model.set_a1("B6", "Date");
    model.set_a1("B7", "10.1.2026"); // DGET needs an exact match

    // Test functions - results should be the same as using empty criteria
    model.set_a1("A9", "=DMIN(A1:C4, C1, A6:A7)");
    model.set_a1("A10", "=DMAX(A1:C4, C1, A6:A7)");
    model.set_a1("A11", "=DAVERAGE(A1:C4, C1, A6:A7)");
    model.set_a1("A12", "=DSUM(A1:C4, C1, A6:A7)");
    model.set_a1("A13", "=DPRODUCT(A1:C4, C1, A6:A7)");
    model.set_a1("A14", "=DGET(A1:C4, C1, B6:B7)");
    model.set_a1("A15", "=DCOUNT(A1:C4, C1, A6:A7)");
    model.set_a1("A16", "=DCOUNTA(A1:C4, C1, A6:A7)");
    model.set_a1("A17", "=DVAR(A1:C4, C1, A6:A7)");
    model.set_a1("A18", "=DVARP(A1:C4, C1, A6:A7)");
    model.set_a1("A19", "=DSTDEV(A1:C4, C1, A6:A7)");
    model.set_a1("A20", "=DSTDEVP(A1:C4, C1, A6:A7)");

    // Test functions with empty criteria - range C6:C7 is empty
    model.set_a1("B9", "=DMIN(A1:C4, C1, C6:C7)");
    model.set_a1("B10", "=DMAX(A1:C4, C1, C6:C7)");
    model.set_a1("B11", "=DAVERAGE(A1:C4, C1, C6:C7)");
    model.set_a1("B12", "=DSUM(A1:C4, C1, C6:C7)");
    model.set_a1("B13", "=DPRODUCT(A1:C4, C1, C6:C7)");
    model.set_a1("B14", "=DGET(A1:C4, C1, B6:B7)");
    model.set_a1("B15", "=DCOUNT(A1:C4, C1, C6:C7)");
    model.set_a1("B16", "=DCOUNTA(A1:C4, C1, C6:C7)");
    model.set_a1("B17", "=DVAR(A1:C4, C1, C6:C7)");
    model.set_a1("B18", "=DVARP(A1:C4, C1, C6:C7)");
    model.set_a1("B19", "=DSTDEV(A1:C4, C1, C6:C7)");
    model.set_a1("B20", "=DSTDEVP(A1:C4, C1, C6:C7)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A9"), *"900");
    assert_eq!(model.get_text_a1("A10"), *"2100");
    assert_eq!(model.get_text_a1("A11"), *"1400");
    assert_eq!(model.get_text_a1("A12"), *"4200");
    assert_eq!(model.get_text_a1("A13"), *"2268000000");
    assert_eq!(model.get_text_a1("A14"), *"1200");
    assert_eq!(model.get_text_a1("A15"), *"3");
    assert_eq!(model.get_text_a1("A16"), *"3");
    assert_eq!(model.get_text_a1("A17"), *"390000");
    assert_eq!(model.get_text_a1("A18"), *"260000");
    assert_eq!(model.get_text_a1("A19"), *"624.4997998");
    assert_eq!(model.get_text_a1("A20"), *"509.9019514");

    assert_eq!(model.get_text_a1("B9"), *"900");
    assert_eq!(model.get_text_a1("B10"), *"2100");
    assert_eq!(model.get_text_a1("B11"), *"1400");
    assert_eq!(model.get_text_a1("B12"), *"4200");
    assert_eq!(model.get_text_a1("B13"), *"2268000000");
    assert_eq!(model.get_text_a1("B14"), *"1200");
    assert_eq!(model.get_text_a1("B15"), *"3");
    assert_eq!(model.get_text_a1("B16"), *"3");
    assert_eq!(model.get_text_a1("B17"), *"390000");
    assert_eq!(model.get_text_a1("B18"), *"260000");
    assert_eq!(model.get_text_a1("B19"), *"624.4997998");
    assert_eq!(model.get_text_a1("B20"), *"509.9019514");
}

// The database and criteria arguments go through `get_reference`, whose
// fallback arm evaluates non-literal references (OFFSET, INDIRECT) in
// reference context. The computed ranges must arrive whole.
#[test]
fn computed_database_and_criteria_references() {
    let mut model = new_empty_model();
    model.set_a1("A1", "ID");
    model.set_a1("B1", "Amount");
    model.set_a1("A2", "1");
    model.set_a1("B2", "1200");
    model.set_a1("A3", "2");
    model.set_a1("B3", "900");
    model.set_a1("A4", "3");
    model.set_a1("B4", "2100");

    model.set_a1("A6", "Amount");
    model.set_a1("A7", ">1000");

    // OFFSET(A1,0,0,4,2) -> A1:B4; INDIRECT("A6:A7") -> the criteria range.
    model.set_a1(
        "D1",
        "=DSUM(OFFSET(A1, 0, 0, 4, 2), B1, INDIRECT(\"A6:A7\"))",
    );
    model.set_a1("D2", "=DCOUNT(OFFSET(A1, 0, 0, 4, 2), B1, A6:A7)");
    model.set_a1("D3", "=DMAX(OFFSET(A1, 0, 0, 4, 2), B1, A6:A7)");
    model.evaluate();

    assert_eq!(model.get_text_a1("D1"), *"3300");
    assert_eq!(model.get_text_a1("D2"), *"2");
    assert_eq!(model.get_text_a1("D3"), *"2100");
}

// Criteria text is not trimmed ("  > 1 " matches nothing: smoke/d-function-criteria J2, excel-graph),
// `<>` is exact (non-text cells match), plain text matches by prefix.
#[test]
fn criteria_grammar_shared_with_countif() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Name");
    model.set_a1("B1", "Qty");
    model.set_a1("A2", "ab");
    model.set_a1("B2", "1");
    model.set_a1("A3", "abc");
    model.set_a1("B3", "2");
    model.set_a1("A4", "xab");
    model.set_a1("B4", "4");
    // Criteria blocks (header + one criterion each)
    model.set_a1("D1", "Name");
    model.set_a1("D2", "ab");
    model.set_a1("F1", "Qty");
    model.set_a1("F2", "  > 1 ");
    model.set_a1("H1", "Name");
    model.set_a1("H2", "<>ab");

    model.set_a1("J1", "=DSUM(A1:B4, \"Qty\", D1:D2)");
    model.set_a1("J2", "=DSUM(A1:B4, \"Qty\", F1:F2)");
    model.set_a1("J3", "=DSUM(A1:B4, \"Qty\", H1:H2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("J1"), *"3");
    assert_eq!(model.get_text_a1("J2"), *"0");
    assert_eq!(model.get_text_a1("J3"), *"6");
}

/// A huge or fractional `field` is out of range, never an overflow (T-1389 security review), and a
/// written empty `field` counts the matching records for DCOUNT and DCOUNTA only
/// (probes/db-argument-modes K12:K13).
#[test]
fn field_edge_cases() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "Name"),
        ("B1", "Qty"),
        ("A2", "a"),
        ("B2", "3"),
        ("A3", "b"),
        ("B3", "4"),
        ("D1", "Qty"),
        ("D2", ">2"),
    ] {
        model.set_a1(cell, value);
    }
    model.set_a1("F1", "=DSUM(A1:B3,1E10,D1:D2)");
    model.set_a1("F2", "=DSUM(A1:B3,2147483647,D1:D2)");
    model.set_a1("F3", "=DSUM(A1:B3,0/0,D1:D2)");
    model.set_a1("F4", "=DCOUNT(A1:B3,,D1:D2)");
    model.set_a1("F5", "=DCOUNTA(A1:B3,,D1:D2)");
    model.set_a1("F6", "=DSUM(A1:B3,,D1:D2)");
    model.set_a1("F7", "=DSUM({1,2;3,4},2,D1:D2)");
    model.set_a1("F8", "=ROWS(DSUM(A1:B3,{1;2},D1:D2))");
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F3"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("F4"), *"2");
    assert_eq!(model.get_text_a1("F5"), *"2");
    assert_eq!(model.get_text_a1("F6"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F7"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F8"), *"2");
}

/// A lifted `field` is charged the criteria table per element, so a large one is refused before
/// it scans (security review of T-1389); the same call without a lift runs.
#[test]
fn lifted_field_is_charged_the_criteria_table() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Name");
    model.set_a1("A2", "a");
    model.set_a1("H1", "Name");
    model.set_a1("H2", "a");
    model.set_a1("F1", "=DCOUNTA(A1:C1000,1,H1:Z1000)");
    model.set_a1("F2", "=ROWS(DCOUNTA(A1:C1000,SEQUENCE(10),H1:Z1000))");
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), *"1");
    assert_eq!(model.get_text_a1("F2"), *"#ERROR!");
}

/// A criteria reference of one row (headers only) is `#VALUE!`, as is a database of one row
/// (probes/db-header-criteria-shapes AB2-AB37, excel-graph).
#[test]
fn one_row_criteria_or_database_is_value_error() {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "Price"), ("A2", "2.5"), ("A3", "4"), ("E1", "Price")] {
        model.set_a1(cell, value);
    }
    model.set_a1("E2", ">3");
    model.set_a1("F1", "=DSUM(A1:A3,\"Price\",E1)");
    model.set_a1("F2", "=DCOUNTA(A1:A3,\"Price\",$E$1)");
    model.set_a1("F3", "=DSUM(A1:A1,\"Price\",E1:E2)");
    model.set_a1("F4", "=DSUM(A1:A3,\"Price\",E1:E2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F4"), *"4");
}

/// Criteria headers name database columns by their text (a number by its coerced text), each
/// criteria column keeps its own header, and a header that names no column makes its conditions
/// computed criteria (probes/db-header-criteria-shapes, probes/db-computed-criteria-shapes).
#[test]
fn criteria_headers_and_computed_criteria() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Name");
    model.set_a1("B1", "Qty");
    model.set_a1("C1", "0.5");
    for row in 2..=6 {
        model.set_a1(&format!("A{row}"), &format!("n{row}"));
        model.set_a1(&format!("B{row}"), &(row - 1).to_string());
        model.set_a1(&format!("C{row}"), &(10 * (row - 1)).to_string());
    }
    // An unknown header over a blank cell is ignored; the second column still tests Qty.
    model.set_a1("H1", "Zzz");
    model.set_a1("I1", "Qty");
    model.set_a1("I2", ">3");
    // The text "0.5" names the number header 0.5.
    model.set_a1("J1", "'0.5");
    model.set_a1("J2", ">20");
    // Computed criteria under a blank or unknown header: TRUE, "TRUE" and 1 hold, ">2" does not.
    model.set_a1("K2", "TRUE");
    model.set_a1("L1", "Zzz");
    model.set_a1("L2", ">2");
    model.set_a1("M1", "Zzz");
    model.set_a1("M2", "'TRUE");
    // A formula condition is evaluated again for each record.
    model.set_a1("N2", "=B2>2");
    // A blank criteria row matches every record.
    model.set_a1("O1", "Qty");
    model.set_a1("O2", ">4");
    model.set_a1("F1", "=DSUM(A1:C6,2,H1:I2)");
    model.set_a1("F2", "=DSUM(A1:C6,2,J1:J2)");
    model.set_a1("F3", "=DSUM(A1:C6,2,K1:K2)");
    model.set_a1("F4", "=DSUM(A1:C6,2,L1:L2)");
    model.set_a1("F5", "=DSUM(A1:C6,2,M1:M2)");
    model.set_a1("F6", "=DSUM(A1:C6,2,N1:N2)");
    model.set_a1("F7", "=DSUM(A1:C6,2,O1:O3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), *"9");
    assert_eq!(model.get_text_a1("F2"), *"12");
    assert_eq!(model.get_text_a1("F3"), *"15");
    assert_eq!(model.get_text_a1("F4"), *"0");
    assert_eq!(model.get_text_a1("F5"), *"15");
    assert_eq!(model.get_text_a1("F6"), *"12");
    assert_eq!(model.get_text_a1("F7"), *"15");
}

/// A formula condition that runs a D-function over its own criteria is bounded (directive 5).
#[test]
fn self_referencing_formula_condition_is_bounded() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Qty");
    for row in 2..=6 {
        model.set_a1(&format!("A{row}"), &row.to_string());
    }
    model.set_a1("E2", "=DSUM($A$1:$A$6,1,$E$1:$E$2)>0");
    model.set_a1("F1", "=DSUM(A1:A6,1,E1:E2)");
    model.evaluate();
    // The nested condition in E2 is not re-evaluated (one level only, a DEVIATION in
    // docs/excel-semantics/statistics.md), so it fails for every record and nothing is summed.
    assert_eq!(model.get_text_a1("F1"), *"0");
}

/// A cell first computed while a formula condition is re-evaluated runs its own D-functions in
/// full: the one-level guard belongs to one formula run (T-1390 fidelity review).
#[test]
fn cells_reached_from_a_formula_condition_evaluate_in_full() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Qty");
    for row in 2..=6 {
        model.set_a1(&format!("A{row}"), &(row - 1).to_string());
        model.set_a1(&format!("K{row}"), "=DCOUNT($A$1:$A$6,1,$G$1:$G$2)");
    }
    model.set_a1("G2", "=A2>2");
    model.set_a1("H2", "=K2>0");
    // The database spans column K so that `=K2>0` reads its record area
    // (probes/db-computed-criteria-cross-sheet: a reference outside the columns matches nothing).
    model.set_a1("M1", "=DCOUNT(A1:K6,1,H1:H2)");
    model.set_a1("N1", "=DCOUNT(A1:A6,1,G1:G2)");
    model.evaluate();
    for row in 2..=6 {
        assert_eq!(model.get_text_a1(&format!("K{row}")), *"3");
    }
    assert_eq!(model.get_text_a1("M1"), *"5");
    assert_eq!(model.get_text_a1("N1"), *"3");
}

/// A record whose moved formula reference lands on the database's header row does not match:
/// over A2:D9 (header row 2) `=B2>3` reads B2 for the first record, and that record fails
/// (gen-m9 4dc761698ca23149 A129, f066968376660329 A292, f61a73cd9af21e6d A289; T-2553).
#[test]
fn formula_condition_on_the_header_row_fails() {
    let mut model = new_empty_model();
    for (row, (qty, price)) in [
        ("Qty", "Price"),
        ("4", "2.5"),
        ("10", "x"),
        ("0", "4"),
        ("2", "TRUE"),
        ("1", "10"),
        ("-4", "0.5"),
        ("7", "100"),
        ("5", "-1"),
    ]
    .into_iter()
    .enumerate()
    {
        model.set_a1(&format!("B{}", row + 1), qty);
        model.set_a1(&format!("C{}", row + 1), price);
    }
    model.set_a1("O1", "Calc");
    model.set_a1("O2", "=B2>3");
    model.set_a1("F1", "=DPRODUCT(B2:C9,2,O1:O2)");
    model.set_a1("F2", "=DMAX(B2:C9,2,O1:O2)");
    model.set_a1("F3", "=DCOUNTA(B2:C9,2,O1:O2)");
    // The same condition over B1:C9 starts at its first record: rows 2, 3, 8 and 9 match.
    model.set_a1("F4", "=DCOUNTA(B1:C9,2,O1:O2)");
    // An absolute row stays put and is not checked: every record reads B2 = 4 and matches.
    model.set_a1("P1", "Calc");
    model.set_a1("P2", "=B$2>3");
    model.set_a1("F5", "=DCOUNTA(B2:C9,2,P1:P2)");
    model.evaluate();
    // Rows 4 (B3 = 10) and 9 (B8 = 7) match; row 3 (B2 = 4, the header row) does not.
    assert_eq!(model.get_text_a1("F1"), *"-4");
    assert_eq!(model.get_text_a1("F2"), *"4");
    assert_eq!(model.get_text_a1("F3"), *"2");
    assert_eq!(model.get_text_a1("F4"), *"4");
    assert_eq!(model.get_text_a1("F5"), *"7");
}

// Excel: a D-function criterion without `=` matches by prefix, wildcard or not, and folds like
// exact MATCH (docs/excel-semantics/criteria.md, probes/t3929-dfunction-wildcard-prefix and
// probes/t3929-wildcard-fold-dfunctions).
#[test]
fn criteria_prefix_wildcards_and_lookup_fold() {
    let mut model = new_empty_model();
    model.set_a1("A1", "k");
    model.set_a1("A2", "abcd");
    model.set_a1("A3", "=UNICHAR(886)&\"x\"");
    model.set_a1("A4", "=UNICHAR(952)&\"x\"");
    model.set_a1("A5", "=UNICHAR(66600)");
    // (criteria column, criterion, expected DCOUNTA over A1:A5)
    let cases = [
        ("C", "a?c", "1"),
        ("D", "=\"=a?c\"", "0"),
        ("E", "?", "4"),
        ("F", "=\"=?\"", "0"),
        ("G", "=UNICHAR(1011)", "1"),
        ("H", "=UNICHAR(1012)", "0"),
        ("I", "=UNICHAR(66560)", "0"),
        ("J", "=UNICHAR(66600)", "1"),
    ];
    for (column, criterion, _) in cases {
        model.set_a1(&format!("{column}1"), "k");
        model.set_a1(&format!("{column}2"), criterion);
        model.set_a1(
            &format!("{column}3"),
            &format!("=DCOUNTA(A1:A5,1,{column}1:{column}2)"),
        );
    }
    model.evaluate();
    for (column, criterion, expected) in cases {
        assert_eq!(
            model.get_text_a1(&format!("{column}3")),
            *expected,
            "{criterion}"
        );
    }
}
