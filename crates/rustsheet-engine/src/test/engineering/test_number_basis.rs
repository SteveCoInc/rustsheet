use crate::test::util::new_empty_model;

#[test]
fn fn_bin2dec() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BIN2DEC(1100100)");
    model.set_a1("A2", "=BIN2DEC(1111111111)");

    model.set_a1("B1", "=BIN2DEC()");
    model.set_a1("B2", "=BIN2DEC(1,2)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "100");
    assert_eq!(model.get_text_a1("A2"), "-1");
    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_bin2hex() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BIN2HEX(11111011, 4)");
    model.set_a1("A2", "=BIN2HEX(1110)");
    model.set_a1("A3", "=BIN2HEX(1111111111)");
    model.set_a1("A4", "=BIN2HEX(1100011011)");

    model.set_a1("B1", "=BIN2HEX()");
    model.set_a1("B2", "=BIN2HEX(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "00FB");
    assert_eq!(model.get_text_a1("A2"), "E");
    assert_eq!(model.get_text_a1("A3"), "FFFFFFFFFF");
    assert_eq!(model.get_text_a1("A4"), "FFFFFFFF1B");
    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_bin2oct() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BIN2OCT(11111011, 4)");
    model.set_a1("A2", "=BIN2OCT(1110)");
    model.set_a1("A3", "=BIN2OCT(1111111111)");
    model.set_a1("A4", "=BIN2OCT(1100011011)");

    model.set_a1("B1", "=BIN2OCT()");
    model.set_a1("B2", "=BIN2OCT(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "0373");
    assert_eq!(model.get_text_a1("A2"), "16");
    assert_eq!(model.get_text_a1("A3"), "7777777777");
    assert_eq!(model.get_text_a1("A4"), "7777777433");
    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_dec2bin() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DEC2BIN(9, 4)");
    model.set_a1("A2", "=DEC2BIN(-100)");
    model.set_a1("A3", "=DEC2BIN(-1)");
    model.set_a1("A4", "=DEC2BIN(0, 3)");

    model.set_a1("B1", "=DEC2BIN()");
    model.set_a1("B2", "=DEC2BIN(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "1001");
    assert_eq!(model.get_text_a1("A2"), "1110011100");
    assert_eq!(model.get_text_a1("A3"), "1111111111");
    assert_eq!(model.get_text_a1("A4"), "000");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_dec2hex() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DEC2HEX(100, 4)");
    model.set_a1("A2", "=DEC2HEX(-54)");
    model.set_a1("A3", "=DEC2HEX(28)");
    model.set_a1("A4", "=DEC2HEX(64, 1)");

    model.set_a1("B1", "=DEC2HEX()");
    model.set_a1("B2", "=DEC2HEX(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "0064");
    assert_eq!(model.get_text_a1("A2"), "FFFFFFFFCA");
    assert_eq!(model.get_text_a1("A3"), "1C");
    assert_eq!(model.get_text_a1("A4"), "#NUM!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_dec2oct() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DEC2OCT(58, 3)");
    model.set_a1("A2", "=DEC2OCT(-100)");

    model.set_a1("B1", "=DEC2OCT()");
    model.set_a1("B2", "=DEC2OCT(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "072");
    assert_eq!(model.get_text_a1("A2"), "7777777634");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_hex2bin() {
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=HEX2BIN("F", 8)"#);
    model.set_a1("A2", r#"=HEX2BIN("B7")"#);
    model.set_a1("A3", r#"=HEX2BIN("FFFFFFFFFF")"#);

    model.set_a1("B1", "=HEX2BIN()");
    model.set_a1("B2", "=HEX2BIN(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "00001111");
    assert_eq!(model.get_text_a1("A2"), "10110111");
    assert_eq!(model.get_text_a1("A3"), "1111111111");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_hex2dec() {
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=HEX2DEC("A5")"#);
    model.set_a1("A2", r#"=HEX2DEC("FFFFFFFF5B")"#);
    model.set_a1("A3", r#"=HEX2DEC("3DA408B9")"#);
    model.set_a1("A4", r#"=HEX2DEC("FE")"#);

    model.set_a1("B1", "=HEX2DEC()");
    model.set_a1("B2", "=HEX2DEC(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "165");
    assert_eq!(model.get_text_a1("A2"), "-165");
    assert_eq!(model.get_text_a1("A3"), "1034160313");
    assert_eq!(model.get_text_a1("A4"), "254");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_hex2oct() {
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=HEX2OCT("F", 3)"#);
    model.set_a1("A2", r#"=HEX2OCT("3B4E")"#);
    model.set_a1("A3", r#"=HEX2OCT("FFFFFFFF00")"#);

    model.set_a1("B1", "=HEX2OCT()");
    model.set_a1("B2", "=HEX2OCT(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "017");
    assert_eq!(model.get_text_a1("A2"), "35516");
    assert_eq!(model.get_text_a1("A3"), "7777777400");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_oct2bin() {
    let mut model = new_empty_model();
    model.set_a1("A1", r"=OCT2BIN(3, 3)");
    model.set_a1("A2", r"=OCT2BIN(7777777000)");

    // bounds
    model.set_a1("G1", r"=OCT2BIN(777)");
    model.set_a1("G2", r"=OCT2BIN(778)");

    model.set_a1("B1", "=OCT2BIN()");
    model.set_a1("B2", "=OCT2BIN(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "011");
    assert_eq!(model.get_text_a1("A2"), "1000000000");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");

    assert_eq!(model.get_text_a1("G1"), "111111111");
    assert_eq!(model.get_text_a1("G2"), "#NUM!");
}

#[test]
fn fn_oct2dec() {
    let mut model = new_empty_model();
    model.set_a1("A1", r"=OCT2DEC(54)");
    model.set_a1("A2", r"=OCT2DEC(7777777533)");

    model.set_a1("B1", "=OCT2DEC()");
    model.set_a1("B2", "=OCT2DEC(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "44");
    assert_eq!(model.get_text_a1("A2"), "-165");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_oct2hex() {
    let mut model = new_empty_model();
    model.set_a1("A1", r"=OCT2HEX(100, 4)");
    model.set_a1("A2", r"=OCT2HEX(7777777533)");

    model.set_a1("B1", "=OCT2HEX()");
    model.set_a1("B2", "=OCT2HEX(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "0040");
    assert_eq!(model.get_text_a1("A2"), "FFFFFFFF5B");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_bin2hex_misc() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BIN2HEX(1100011011, -2)");
    model.set_a1("A2", "=BIN2HEX(1100011011, 11)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
}

#[test]
fn fn_bin2oct_misc() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BIN2OCT(1100011011, -2)");
    model.set_a1("A2", "=BIN2OCT(1100011011, 11)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
}

#[test]
fn fn_dec2oct_misc() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DEC2OCT(-1213, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"7777775503");
}

#[test]
fn fn_dec2bin_misc() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DEC2BIN(-511, 4)");
    model.set_a1("A2", "=DEC2BIN(TRUE, -1)");
    model.set_a1("A3", "=DEC2OCT(TRUE, -1)");
    model.set_a1("A4", "=DEC2HEX(TRUE, -1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"1000000001");
    // Excel's #NUM!: the invalid places is checked before the boolean number (T-2545,
    // probes/t2545-base-places-boolean A1:A3; this test pinned our old #VALUE!).
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
}

#[test]
fn fn_hex2whatever_misc() {
    let mut model = new_empty_model();
    model.set_a1("A1", r"=HEX2BIN(TRUE, 4)");
    model.set_a1("A2", r"=HEX2DEC(TRUE)");
    model.set_a1("A3", r"=HEX2OCT(TRUE, 4)");
    // HEX2DEC takes one argument (the lifting table, T-1383), as Excel does.
    model.set_a1("A4", r"=HEX2DEC(TRUE, 4)");

    model.evaluate();
    // A boolean is #VALUE! (probes/eng-leading-plus-oct-bool A35:A45)
    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#VALUE!");
}

#[test]
fn fn_oct2whatever_misc() {
    let mut model = new_empty_model();
    model.set_a1("A1", r"=OCT2BIN(TRUE, 4)");
    model.set_a1("A2", r"=OCT2DEC(TRUE)");
    model.set_a1("A3", r"=OCT2HEX(TRUE, 4)");
    // OCT2DEC takes one argument (the lifting table, T-1383), as Excel does.
    model.set_a1("A4", r"=OCT2DEC(TRUE, 4)");

    model.evaluate();
    // A boolean is #VALUE! (probes/eng-leading-plus-oct-bool A35:A45)
    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#VALUE!");
}

#[test]
fn fn_oct2dec_misc() {
    let mut model = new_empty_model();
    model.set_a1("A1", r"=OCT2DEC(777)");
    model.set_a1("A2", r#"=OCT2DEC("777")"#);
    model.set_a1("A3", r#"=OCT2DEC("-1")"#);
    model.set_a1("A4", r#"=OCT2BIN("-1")"#);
    model.set_a1("A5", r#"=OCT2HEX("-1")"#);
    model.set_a1("A6", r"=OCT2DEC(4000000000)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"511");
    assert_eq!(model.get_text_a1("A1"), *"511");

    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A5"), *"#NUM!");
    assert_eq!(model.get_text_a1("A6"), *"-536870912");
}

/// Excel: a negative result is ten digits whatever a valid `places` says
/// (probes/eng-gen-sample-edges C1:C6, C11).
#[test]
fn negative_hex_result_ignores_places() {
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=HEX2OCT("FFFFFFFFFF",4)"#);
    model.set_a1("A2", r#"=HEX2BIN("FFFFFFFFFF",4)"#);
    model.set_a1("A3", r#"=HEX2OCT("FFFFFFFFFF",11)"#);
    model.set_a1("A4", r#"=HEX2BIN("FFFFFFFFFF",0)"#);
    model.set_a1("A5", r#"=HEX2OCT("1F",1)"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"7777777777");
    assert_eq!(model.get_text_a1("A2"), *"1111111111");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A5"), *"#NUM!");
}

#[test]
fn oct2dec_of_eleven_digits_is_num() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=OCT2DEC(77777777777)");
    model.set_a1("A2", "=OCT2DEC(7777777777)");
    model.set_a1("A3", r#"=IMABS("+i")"#);
    model.set_a1("A4", r#"=IMABS("++i")"#);
    model.set_a1("A5", r#"=IMSUB("+i","1")"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"-1");
    assert_eq!(model.get_text_a1("A3"), *"1");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A5"), *"-1+i");
}

#[test]
fn bin2_of_non_numeric_text_is_num() {
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=BIN2DEC("abc")"#);
    model.set_a1("A2", r#"=BIN2HEX(" x ")"#);
    model.set_a1("A3", r#"=BIN2OCT("abc",4)"#);
    model.set_a1("A4", "=BIN2DEC(TRUE)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"#VALUE!");
}

#[test]
fn dec2_of_non_numeric_text_stays_value() {
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=DEC2OCT("abc")"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
}

#[test]
fn an_invalid_places_is_num_before_the_number_is_read() {
    // gen-m9 T-2258 triage (excel-graph; T-2545): places is range-checked before the number, so
    // an invalid one beats the number's error or unreadable text; a valid one does not.
    let mut model = new_empty_model();
    let cells = [
        ("A1", "=DEC2BIN(NA(),0)", "#NUM!"),
        ("A2", "=DEC2BIN(\"x\",11)", "#NUM!"),
        ("A3", "=HEX2OCT(NA(),0.5)", "#NUM!"),
        ("A4", "=DEC2BIN(NA(),3)", "#N/A"),
        ("A5", "=BIN2OCT(\"ABC\",-1)", "#NUM!"),
        ("A6", "=OCT2HEX(1/0,-0.5)", "#NUM!"),
        ("A7", "=DEC2OCT(NA(),Z99)", "#NUM!"),
    ];
    for (cell, formula, _) in cells {
        model.set_a1(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model.get_text_a1(cell), expected, "{formula}");
    }
}

#[test]
fn inumber_currency_sign_and_end_whitespace() {
    // probes/eng-inumber-currency-shapes A27:A40 and A13:A26 (excel-graph).
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=IMABS("- $1+$2i")=SQRT(5)"#);
    model.set_a1("A2", r#"=IMABS("$.5")"#);
    model.set_a1("A3", r#"=IMABS("1.$5")"#);
    model.set_a1("A4", r#"=IMABS("£1+2i")"#);
    model.set_a1("A5", r#"=IMABS(" "&CHAR(9)&"5")"#);
    model.set_a1("A6", r#"=IMABS("5"&UNICHAR(160))"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"0.5");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A5"), *"#NUM!");
    assert_eq!(model.get_text_a1("A6"), *"#NUM!");
}
