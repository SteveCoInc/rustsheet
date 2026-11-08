#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Brian Wiles");
    model.set_a1("A2", "Jeden,dwa,trzy,cztery");

    model.set_a1("B1", "=TEXTAFTER(A1, \" \")");
    model.set_a1("B2", "=TEXTAFTER(A2, \",\")");
    model.set_a1("C2", "=TEXTAFTER(A2, \",\", 2)");

    model.set_a1("H1", "=TEXTBEFORE(A1, \" \")");
    model.set_a1("H2", "=TEXTBEFORE(A2, \",\")");
    model.set_a1("I2", "=_xlfn.TEXTBEFORE(A2, \",\", 2)");

    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"Wiles");
    assert_eq!(model.get_text_a1("B2"), *"dwa,trzy,cztery");
    assert_eq!(model.get_text_a1("C2"), *"trzy,cztery");

    assert_eq!(model.get_text_a1("H1"), *"Brian");
    assert_eq!(model.get_text_a1("H2"), *"Jeden");
    assert_eq!(model.get_text_a1("I2"), *"Jeden,dwa");
    assert_eq!(model.get_formula_a1("I2"), *"=TEXTBEFORE(A2, \",\", 2)");
}

// T-1796: match_mode 1 once located the delimiter in `to_lowercase()` of the text and sliced the
// original at those byte offsets; İ lowercases to two chars, so the offsets ran past the end.
#[test]
fn test_textbefore_textafter_case_insensitive_non_ascii_length_change() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"İİİ,a\",\",\",1,1)");
    model.set_a1("A2", "=TEXTAFTER(\"İİİ,a\",\",\",1,1)");
    model.set_a1("A3", "=TEXTBEFORE(\"İİİ,a,b\",\",\",-1,1)");
    model.set_a1("A4", "=TEXTAFTER(\"İİİ,a,b\",\",\",-1,1)");
    // Kelvin sign (3 bytes) lowercases to ASCII k (1 byte), shifting offsets the other way.
    model.set_a1("A5", "=TEXTBEFORE(\"\u{212A}\u{212A},b\",\",\",1,1)");
    model.set_a1("A6", "=TEXTAFTER(\"\u{212A}\u{212A},b\",\",\",-1,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "İİİ");
    assert_eq!(model.get_text_a1("A2"), "a");
    assert_eq!(model.get_text_a1("A3"), "İİİ,a");
    assert_eq!(model.get_text_a1("A4"), "b");
    assert_eq!(model.get_text_a1("A5"), "\u{212A}\u{212A}");
    assert_eq!(model.get_text_a1("A6"), "b");
}

// Excel (probes/textbefore-casefold-length): same folding as TEXTSPLIT: İ matches "i" (either
// way round), the Kelvin sign does not match "k", ß / ẞ do not match "ss", Ä matches ä.
#[test]
fn test_textbefore_textafter_case_insensitive_matches_excel_folding() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"aİbİc\",\"i\",1,1)");
    model.set_a1("A2", "=TEXTAFTER(\"aİbİc\",\"i\",1,1)");
    model.set_a1("A3", "=TEXTBEFORE(\"aibic\",\"İ\",1,1)");
    model.set_a1("A4", "=TEXTBEFORE(\"a\u{212A}bkc\",\"k\",1,1)");
    model.set_a1("A5", "=TEXTBEFORE(\"akbkc\",\"\u{212A}\",1,1)");
    model.set_a1("A6", "=TEXTBEFORE(\"aßbSSc\",\"ss\",1,1)");
    model.set_a1("A7", "=TEXTBEFORE(\"assbẞc\",\"ß\",1,1)");
    model.set_a1("A8", "=TEXTBEFORE(\"ÄxäyÄ\",\"ä\",2,1)");
    model.set_a1("A9", "=TEXTAFTER(\"ÄxäyÄ\",\"ä\",2,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "a");
    assert_eq!(model.get_text_a1("A2"), "bİc");
    assert_eq!(model.get_text_a1("A3"), "a");
    assert_eq!(model.get_text_a1("A4"), "a\u{212A}b");
    assert_eq!(model.get_text_a1("A5"), "#N/A");
    assert_eq!(model.get_text_a1("A6"), "aßb");
    assert_eq!(model.get_text_a1("A7"), "#N/A");
    assert_eq!(model.get_text_a1("A8"), "Äx");
    assert_eq!(model.get_text_a1("A9"), "yÄ");
}

// Excel (probes/textbefore-casefold-length A17-A25): sigma folds by position, not to a single
// letter: a Σ followed by a letter is σ, any other Σ is ς, and ς matches only ς.
#[test]
fn test_textbefore_textafter_case_insensitive_final_sigma() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"aΟΣb\",\"ος\",1,1)");
    model.set_a1("A2", "=TEXTAFTER(\"aΟΣb\",\"ς\",1,1)");
    model.set_a1("A3", "=TEXTBEFORE(\"aοςb\",\"Σ\",1,1)");
    model.set_a1("A4", "=TEXTBEFORE(\"aοςb\",\"ς\",1,1)");
    model.set_a1("A5", "=TEXTBEFORE(\"aοσb\",\"ς\",1,1)");
    model.set_a1("A6", "=TEXTBEFORE(\"aοσb\",\"Σ\",1,1)");
    model.set_a1("A7", "=TEXTBEFORE(\"aοςb\",\"σ\",1,1)");
    model.set_a1("A8", "=TEXTBEFORE(\"aΟΣb\",\"σ\",1,1)");
    model.set_a1("A9", "=TEXTBEFORE(\"aΟΣ\",\"ς\",1,1)");
    model.evaluate();
    for cell in ["A1", "A2", "A5", "A6", "A7"] {
        assert_eq!(model.get_text_a1(cell), "#N/A", "{cell}");
    }
    assert_eq!(model.get_text_a1("A3"), "aο");
    assert_eq!(model.get_text_a1("A4"), "aο");
    assert_eq!(model.get_text_a1("A8"), "aΟ");
    assert_eq!(model.get_text_a1("A9"), "aΟ");
}

// T-4473 (probes/t4473-textsplit-fold-gaps A8, A29, A35, A105, A107): the delimiter `İ` matches
// `I`, the Angstrom sign matches neither `å` nor `Å`, and a `Σ` before a combining mark is `σ`.
#[test]
fn test_textbefore_case_insensitive_dotted_i_angstrom_and_sigma_before_mark() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"aIb\",\"İ\",1,1)");
    model.set_a1("A2", "=TEXTBEFORE(\"a\u{212B}b\",\"å\",1,1)");
    model.set_a1("A3", "=TEXTBEFORE(\"a\u{212B}b\",\"Å\",1,1)");
    model.set_a1("A4", "=TEXTBEFORE(\"ΟΣ\u{301}x\",\"σ\",1,1)");
    model.set_a1("A5", "=TEXTBEFORE(\"ΟΣ\u{301}x\",\"ς\",1,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "a");
    assert_eq!(model.get_text_a1("A2"), "#N/A");
    assert_eq!(model.get_text_a1("A3"), "#N/A");
    assert_eq!(model.get_text_a1("A4"), "Ο");
    assert_eq!(model.get_text_a1("A5"), "#N/A");
}

// A huge negative instance number saturates to i32::MIN in the caller; negating it must not
// overflow (T-1796 security review). It behaves like the largest representable negative count.
#[test]
fn test_textbefore_textafter_huge_negative_instance_does_not_overflow() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"a,b\",\",\",-1E10)");
    model.set_a1("A2", "=TEXTBEFORE(\"a,b\",\",\",-1E10,1)");
    model.set_a1("A3", "=TEXTAFTER(\"a,b\",\",\",-1E10)");
    model.set_a1("A4", "=TEXTAFTER(\"a,b\",\",\",-1E10,1)");
    model.set_a1("B1", "=TEXTBEFORE(\"a,b\",\",\",-2147483647)");
    model.set_a1("B2", "=TEXTBEFORE(\"a,b\",\",\",-2147483647,1)");
    model.set_a1("B3", "=TEXTAFTER(\"a,b\",\",\",-2147483647)");
    model.set_a1("B4", "=TEXTAFTER(\"a,b\",\",\",-2147483647,1)");
    model.evaluate();
    for row in 1..=4 {
        assert_eq!(
            model.get_text_a1(&format!("A{row}")),
            model.get_text_a1(&format!("B{row}")),
            "row {row}"
        );
    }
}

// T-1272: the text lifts over a range, the delimiter is a list (probes/m7-argument-modes F13-K13).
#[test]
fn text_lifts_and_delimiters_are_a_list() {
    let mut model = new_empty_model();
    model.set_a1("A1", "a,b");
    model.set_a1("A2", "c-d");
    model.set_a1("A3", "e,f");
    model.set_a1("C1", "=TEXTBEFORE(A1:A3,{\",\";\"-\"})");
    model.set_a1("D1", "=TEXTAFTER(A1:A3,{\",\";\"-\"})");
    model.set_a1("E1", "=TEXTBEFORE(\"a,b,c\",\",\",{1;2})");
    model.set_a1("F1", "=TEXTSPLIT(A1:A3,\",\")");
    model.set_a1("G1", "=REGEXTEST(A1:A3,\"a\")");
    model.set_a1("H1", "=REGEXEXTRACT(\"a1b2\",{\"[0-9]\";\"[a-z]\"})");
    model.set_a1("I1", "=REGEXREPLACE(A1:A3,\",\",\"+\")");
    model.evaluate();

    let column = |letter: char, rows: i32| -> Vec<String> {
        let c = i32::from(letter as u8 - b'A') + 1;
        (1..=rows)
            .map(|r| model.get_text_by_index(0, r, c))
            .collect()
    };
    assert_eq!(column('C', 3), ["a", "c", "e"]);
    assert_eq!(column('D', 3), ["b", "d", "f"]);
    assert_eq!(column('E', 2), ["a", "a,b"]);
    assert_eq!(column('F', 3), ["a", "c-d", "e"]);
    assert_eq!(column('G', 3), ["TRUE", "FALSE", "FALSE"]);
    assert_eq!(column('H', 2), ["1", "a"]);
    assert_eq!(column('I', 3), ["a+b", "c-d", "e+f"]);
}

// match_end and if_not_found with a delimiter list and every list entry longer than the text
// (probes/textsplit-lift-guesses A10-A21, T-1824; the error inside the list, A5, is
// probes/textbefore-delimiter-list-elements A1, T-1361). See docs/excel-semantics/text.md
// "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar arguments".
#[test]
fn list_with_match_end_and_if_not_found() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"a,b\",{\"-\",\";\"},1,0,1)");
    model.set_a1("A2", "=TEXTAFTER(\"a,b\",{\"-\",\";\"},1,0,1)");
    model.set_a1("A3", "=TEXTAFTER(\"a,b\",{\"-\",\";\"},1,0,0,\"none\")");
    model.set_a1("A4", "=TEXTAFTER(\"a,b\",{\"-\",\";\"})");
    model.set_a1("A5", "=TEXTAFTER(\"a,b\",{\",\",#N/A})");
    model.set_a1("A6", "=TEXTAFTER(\"a\",{\"abc\",\"abcd\"})");
    model.set_a1("A7", "=TEXTAFTER(\"a\",{\"abc\",\"abcd\"},1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"a,b");
    assert_eq!(model.get_text_a1("A2"), *"");
    assert_eq!(model.get_text_a1("A3"), *"none");
    assert_eq!(model.get_text_a1("A4"), *"#N/A");
    assert_eq!(model.get_text_a1("A5"), *"#N/A");
    assert_eq!(model.get_text_a1("A6"), *"#N/A");
    assert_eq!(model.get_text_a1("A7"), *"#VALUE!");
}

// A per-element REGEX compile error stays in its element (probes/textsplit-lift-guesses A22-A27;
// docs/excel-semantics/text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift their text and
// scalar arguments").
#[test]
fn regex_pattern_errors_stay_per_element() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=REGEXTEST(\"abc\",{\"a\";\"(\"})");
    model.set_a1("C1", "=REGEXTEST({\"a\";\"b\"},\"(\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("C1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("C2"), *"#VALUE!");
}

// A call that does not lift is capped too: 32,768 positions times 10,000 delimiters is past
// `MAX_LIFTED_CELL_VISITS` (DEVIATION, decision 0014: Excel has no such limit). T-1272.
#[test]
fn unlifted_call_over_a_long_delimiter_list_is_capped() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(REPT(\"a\",32767),SEQUENCE(10000))");
    model.set_a1(
        "A2",
        "=TEXTBEFORE(\"a-b\",IF(SEQUENCE(100)=1,\"-\",\"zz\"))",
    );
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"a");
}

// Ordinary patterns compile as on main: the cache bounds memory by its entry count, not by a
// per-pattern size below the regex crate's default (T-1272 review).
#[test]
fn ordinary_counted_patterns_still_compile() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=REGEXTEST(REPT(\"a\",100),\"\\w{100}\")");
    model.set_a1("A2", "=REGEXTEST(REPT(\"a\",30),\"(\\w+){30}\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
}

// One long cell in a mostly blank column costs its own length once, not once per element
// (T-1272 review): Excel gives 100000 for both.
#[test]
fn one_long_cell_does_not_cap_a_long_lifted_column() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=REPT(\"a,\",1000)");
    model.set_a1("C1", "=ROWS(TEXTBEFORE(A1:A100000,\",\"))");
    model.set_a1("D1", "=ROWS(REGEXTEST(A1:A100000,\",\"))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"100000");
    assert_eq!(model.get_text_a1("D1"), *"100000");
}

// T-1272 performance review: a delimiter's bytes count, not only the delimiter, so 512
// delimiters of 16,384 bytes over a 32,767-character text are capped (DEVIATION, decision
// 0014), while one long delimiter still matches.
#[test]
fn long_delimiters_count_their_bytes() {
    let mut model = new_empty_model();
    model.set_a1(
        "A1",
        "=TEXTBEFORE(REPT(\"a\",32767),REPT(\"a\",16383)&\"b\"&SEQUENCE(512))",
    );
    model.set_a1(
        "A2",
        "=LEN(TEXTBEFORE(REPT(\"a\",32766)&\"b\",REPT(\"a\",16000)&\"b\"))",
    );
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"16766");
}

// The delimiter list is folded once per call, not once per element: match_mode 1 over a lifted
// text with a long list (T-1272 performance review).
#[test]
fn case_insensitive_list_over_a_lifted_text() {
    let mut model = new_empty_model();
    model.set_a1(
        "A1",
        "=TEXTBEFORE(SEQUENCE(20)&\"X\",VSTACK(\"x\",REPT(\"a\",1000)&SEQUENCE(4000)),1,1)",
    );
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"1");
    assert_eq!(model.get_text_a1("A20"), *"20");
}

// T-1361: a delimiter range of any shape is a list; numbers and booleans are their text, a blank
// cell is the empty delimiter, which decides the result alone (probes/textbefore-delimiter-range,
// probes/textbefore-empty-delimiter-list, probes/textbefore-delimiter-list-elements).
#[test]
fn delimiter_ranges_are_lists() {
    let mut model = new_empty_model();
    model.set_a1("D1", "x");
    model.set_a1("E1", "y");
    model.set_a1("D2", "z");
    model.set_a1("E2", ",");
    model.set_a1("D3", "1");
    model.set_a1("E3", "TRUE");
    model.set_a1("E4", "q");
    model.set_a1("D5", "=NA()");
    model.set_a1("E5", "b");
    model.set_a1("A1", "=TEXTBEFORE(\"a,b;c\",D1:E2)");
    model.set_a1("A2", "=TEXTBEFORE(\"a1b\",D3:E3)");
    model.set_a1("A3", "=TEXTAFTER(\"a TRUE b\",D3:E3)");
    model.set_a1("A4", "=TEXTBEFORE(\"a true b\",D3:E3,,1)");
    model.set_a1("A5", "=TEXTBEFORE(\"xyz\",D4:E4)");
    model.set_a1("A6", "=TEXTAFTER(\"xyz\",D4:E4)");
    model.set_a1("A7", "=TEXTBEFORE(\"xyz\",D4:E4,-1)");
    model.set_a1("A8", "=TEXTBEFORE(\"abc\",D5:E5)");
    model.set_a1("A9", "=TEXTBEFORE(\"aFALSEb\",{FALSE,\"q\"})");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"a");
    assert_eq!(model.get_text_a1("A2"), *"a");
    assert_eq!(model.get_text_a1("A3"), *" b");
    assert_eq!(model.get_text_a1("A4"), *"a ");
    assert_eq!(model.get_text_a1("A5"), *"");
    assert_eq!(model.get_text_a1("A6"), *"xyz");
    assert_eq!(model.get_text_a1("A7"), *"xyz");
    assert_eq!(model.get_text_a1("A8"), *"#N/A");
    assert_eq!(model.get_text_a1("A9"), *"a");
}

// T-1361: with the empty delimiter the result ignores the other delimiters and instance_num's
// size; a one-UTF-16-unit text matches at its end (probes/textbefore-empty-delimiter-list,
// probes/textbefore-delimiter-list-elements A8-A21).
#[test]
fn empty_delimiter_matches_at_start_or_end() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"abc\",\"\",2)");
    model.set_a1("A2", "=TEXTBEFORE(\"42\",{\"\",\"q\"},1E+15)");
    model.set_a1("A3", "=TEXTBEFORE(\"abc\",{\"b\",\"\"},-1)");
    model.set_a1("A4", "=TEXTBEFORE(\"2\",\"\")");
    model.set_a1("A5", "=TEXTAFTER(\"q\",{\"\",\"q\"},-1)");
    model.set_a1("A6", "=TEXTBEFORE(\"😀\",\"\")");
    model.set_a1("A7", "=TEXTBEFORE(\"\",{\"\",\"q\"},2)");
    model.set_a1("A8", "=TEXTBEFORE(\"abc\",\"\",0)");
    model.set_a1("A9", "=TEXTBEFORE(\"abc\",\"\",1,0,2)");
    model.set_a1("A10", "=TEXTBEFORE(\"a\",\"\",,,,\"nf\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"");
    assert_eq!(model.get_text_a1("A2"), *"");
    assert_eq!(model.get_text_a1("A3"), *"abc");
    assert_eq!(model.get_text_a1("A4"), *"2");
    assert_eq!(model.get_text_a1("A5"), *"");
    assert_eq!(model.get_text_a1("A6"), *"");
    assert_eq!(model.get_text_a1("A7"), *"");
    assert_eq!(model.get_text_a1("A8"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A9"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A10"), *"a");
}

// T-1799: probes/textbefore-sigma-contexts (excel-graph; the probe wraps cells in IFERROR, so an error there is #N/A by the delimiter-not-found rule): a capital sigma is `σ` before a
// letter or digit and `ς` otherwise, and a lone delimiter `Σ` is `ς` even against a text `Σ`.
#[test]
fn test_textbefore_textafter_sigma_contexts() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"aΣb\",\"Σ\",1,1)");
    model.set_a1("A2", "=TEXTBEFORE(\"ΟΔΟΣ ΚΑΙ\",\"ς\",1,1)");
    model.set_a1("A3", "=TEXTBEFORE(\"ΟΔΟΣ ΚΑΙ\",\"σ\",1,1)");
    model.set_a1("A4", "=TEXTBEFORE(\"ΟΔΟΣ1\",\"σ\",1,1)");
    model.set_a1("A5", "=TEXTBEFORE(\"ΟΔΟΣ1\",\"ς\",1,1)");
    model.set_a1("A6", "=TEXTAFTER(\"ΟΔΟΣ,Α\",\"ς\",1,1)");
    model.set_a1("A7", "=TEXTAFTER(\"ΟΔΟΣ,Α\",\"σ\",1,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#N/A");
    assert_eq!(model.get_text_a1("A2"), *"ΟΔΟ");
    assert_eq!(model.get_text_a1("A3"), *"#N/A");
    assert_eq!(model.get_text_a1("A4"), *"ΟΔΟ");
    assert_eq!(model.get_text_a1("A5"), *"#N/A");
    assert_eq!(model.get_text_a1("A6"), *",Α");
    assert_eq!(model.get_text_a1("A7"), *"#N/A");
}

// T-2314: probes/t2314-textbefore-instance-bound (excel-graph).
#[test]
fn test_textbefore_textafter_instance_num_past_the_text() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"b\",\"a\",-2,1,1)");
    model.set_a1("A2", "=TEXTBEFORE(\"ab\",\"a\",3,0,0,\"nf\")");
    model.set_a1("A3", "=TEXTBEFORE(\"ab\",\"a\",2)");
    model.set_a1("A4", "=TEXTAFTER(\"😀\",\"a\",2)");
    model.set_a1("A5", "=TEXTAFTER(\"😀\",\"a\",3)");
    model.set_a1("A6", "=TEXTAFTER(\"é\",\"a\",2)");
    model.set_a1("A7", "=TEXTBEFORE(\"abc\",\"\",2)");
    model.set_a1("A8", "=TEXTBEFORE(\"a,b\",\",\",2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A3"), *"#N/A");
    assert_eq!(model.get_text_a1("A4"), *"#N/A");
    assert_eq!(model.get_text_a1("A5"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A6"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A7"), *"");
    assert_eq!(model.get_text_a1("A8"), *"#N/A");
}

#[test]
fn test_textbefore_textafter_match_mode_values() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"aBc\",\"b\",1,2)");
    model.set_a1("A2", "=TEXTBEFORE(\"aBc\",\"b\",1,-1)");
    model.set_a1("A3", "=TEXTBEFORE(\"aBc\",\"b\",1,0.5)");
    model.set_a1("A4", "=TEXTBEFORE(\"aBc\",\"b\",1,1.5)");
    model.set_a1("A5", "=TEXTBEFORE(\"aBc\",\"b\",1,TRUE)");
    model.set_a1("A6", "=TEXTBEFORE(\"abc\",\"\",2,123456789.123)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A3"), *"#N/A");
    assert_eq!(model.get_text_a1("A4"), *"a");
    assert_eq!(model.get_text_a1("A5"), *"a");
    assert_eq!(model.get_text_a1("A6"), *"#VALUE!");
}

#[test]
fn test_textbefore_textafter_number_if_not_found_is_text() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"b\",\"a\",1,0,0,5)");
    model.set_a1("A2", "=TEXTBEFORE(\"b\",\"a\",1,0,0,-2.5)");
    model.set_a1("A3", "=TEXTAFTER(\"b\",\"a\",1,0,0,TRUE)");
    model.set_a1("A4", "=ISTEXT(TEXTBEFORE(\"b\",\"a\",1,0,0,5))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"5");
    assert_eq!(model.get_text_a1("A2"), *"-2.5");
    assert_eq!(model.get_text_a1("A3"), *"TRUE");
    assert_eq!(model.get_text_a1("A4"), *"TRUE");
}

// match_end is cut toward zero like match_mode: -0.0001 and 0.5 are 0, 1 matches the end of the
// text (gen-m7/2d9b48c6363e7a16 A660, T-2412).
#[test]
fn test_textafter_match_end_is_cut_toward_zero() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTAFTER(\"'a\",\"zz\",1,1,-0.0001)");
    model.set_a1("A2", "=TEXTAFTER(\"'a\",\"zz\",1,1,0.5)");
    model.set_a1("A3", "=TEXTAFTER(\"'a\",\"zz\",1,1,1)");
    model.set_a1("A4", "=TEXTBEFORE(\"'a\",\"zz\",1,1,-0.0001)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#N/A");
    assert_eq!(model.get_text_a1("A2"), *"#N/A");
    assert_eq!(model.get_text_a1("A3"), *"");
    assert_eq!(model.get_text_a1("A4"), *"#N/A");
}

// The every-delimiter-longer check counts UTF-16 units, not UTF-8 bytes: "é" is one unit like
// "a", so `("a","é",1)` reaches the ordinary not-found `#N/A`, and "é" is shorter than "ab", so
// the two-argument `("é","ab")` is `#N/A` and the three-argument one `#VALUE!`
// (probes/t4474-textbefore-utf16-bounds B1, A2, B2, T-4474).
#[test]
fn test_textbefore_lengths_are_utf16_units() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTBEFORE(\"a\",\"é\",1)");
    model.set_a1("A2", "=TEXTBEFORE(\"é\",\"ab\")");
    model.set_a1("A3", "=TEXTBEFORE(\"é\",\"ab\",1)");
    model.set_a1("A4", "=TEXTBEFORE(\"😀\",\"abc\",1)");
    model.set_a1("A5", "=TEXTBEFORE(\"😀\",\"ab\",1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#N/A");
    assert_eq!(model.get_text_a1("A2"), *"#N/A");
    assert_eq!(model.get_text_a1("A3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A4"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A5"), *"#N/A");
}
