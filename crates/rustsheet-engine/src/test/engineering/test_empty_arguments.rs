//! A written-empty argument of the Analysis `ToolPak` engineering functions: a required one (and
//! ERF's `upper_limit`) is `#N/A`, an empty places, step or suffix is omitted
//! (probes/t4872-atp-empty-argument, excel-graph), and a call past the maximum argument count is
//! still `#ERROR!`, empty or not (syntax.md "Minimum argument counts"; T-4872).

use crate::test::util::new_empty_model;

#[test]
fn empty_arguments_follow_the_toolpak_rule() {
    let mut model = new_empty_model();
    let cases = [
        ("COMPLEX(1,)", "#N/A"),
        ("COMPLEX(1,1,)", "1+i"),
        ("ERF(6,)", "#N/A"),
        ("DEC2BIN(5,)", "101"),
        ("GESTEP(1,)", "1"),
        ("LET(e,,GESTEP(1,e))", "1"),
        ("LAMBDA(a,b,DEC2BIN(a,b))(,2)", "#N/A"),
        ("DEC2BIN(5,,)", "#ERROR!"),
        ("COMPLEX(1,1,,)", "#ERROR!"),
        ("CONVERT(1,\"m\",\"ft\",)", "#ERROR!"),
        ("GESTEP(1,,)", "#ERROR!"),
        ("ERF(1,2,)", "#ERROR!"),
        // The #N/A comes at the empty argument's own turn, and per element
        // (probes/t4872-r2-order-and-lift).
        ("COMPLEX(1/0,)", "#DIV/0!"),
        ("ERF(1/0,)", "#DIV/0!"),
        ("DEC2BIN(,1/0)", "#DIV/0!"),
        ("ROWS(COMPLEX({1;2},))", "2"),
    ];
    for (row, (formula, _)) in (1..).zip(cases) {
        model.set_a1(&format!("A{row}"), &format!("={formula}"));
    }
    model.evaluate();
    for (row, (formula, expected)) in (1..).zip(cases) {
        assert_eq!(model.get_text_a1(&format!("A{row}")), expected, "{formula}");
    }
}
