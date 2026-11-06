//! Excel's text order (`functions::collation`; probes/text-collation-order, T-0220).

use std::cmp::Ordering;

use crate::functions::util::collation::{
    cmp_text, cmp_text_excel, cmp_text_pairwise, collation_key,
};
use crate::text::Text;

/// Asserts `texts` are strictly increasing in Excel's order.
fn assert_increasing(texts: &[&str]) {
    for pair in texts.windows(2) {
        assert_eq!(
            cmp_text_excel(pair[0], pair[1]),
            Ordering::Less,
            "{:?} < {:?}",
            pair[0],
            pair[1]
        );
        assert_eq!(cmp_text_excel(pair[1], pair[0]), Ordering::Greater);
    }
}

#[test]
fn punctuation_digits_letters() {
    assert_increasing(&[
        "", "'", "''", "-", "--", " ", "  ", "\u{a0}", "!", "\"", "#", "$", "%", "&", "(", ")",
        "*", ",", ".", "/", ":", ";", "?", "@", "[", "\\", "]", "^", "_", "`", "{", "|", "}", "~",
        "+", "<", "=", ">", "0", "1", "9", "a", "b", "z",
    ]);
}

#[test]
fn hyphen_and_apostrophe_break_ties_only() {
    assert_increasing(&["1", "'1", "-1", "2"]);
    assert_increasing(&["a", "a'", "a-", "'a", "-a", "aa"]);
    assert_increasing(&["ab", "a'b", "a-b", "æ", "b"]);
    assert_increasing(&["co op", "coop", "co-op"]);
    // gen-m2/96a0952412cefa6c A284: punctuation before digits.
    assert_increasing(&["?b", "1.5"]);
}

#[test]
fn accents_are_a_later_tiebreak() {
    assert_increasing(&["a", "á", "à", "â", "ä", "å", "aa"]);
    assert_increasing(&["e", "é", "f"]);
    assert_increasing(&["o", "ö", "ø", "p"]);
    assert_increasing(&["resume", "résumé"]);
    assert_increasing(&["y", "ÿ", "z"]);
    // gen-m2/2cfe279fe77a000d A318: "ü" sorts with "u", before "x".
    assert_increasing(&["u", "ü", "x"]);
}

#[test]
fn case_and_expansions_are_equal() {
    for (left, right) in [
        ("a", "A"),
        ("Resume", "resume"),
        ("Résumé", "résumé"),
        ("À", "à"),
        ("æ", "ae"),
        ("Æ", "AE"),
        ("ß", "ss"),
        ("ß", "SS"),
    ] {
        assert_eq!(
            cmp_text_excel(left, right),
            Ordering::Equal,
            "{left:?} = {right:?}"
        );
    }
    assert_ne!(cmp_text_excel("a-b", "ab"), Ordering::Equal);
    assert_ne!(cmp_text_excel("\u{a0}1", " 1"), Ordering::Equal);
}

#[test]
fn other_characters_keep_code_point_order_after_letters() {
    assert_increasing(&["\u{1}", " ", "z", "α", "β", "ω", "😀"]);
}

/// probes/text-collation-unprobed (T-1590, excel-graph): the whole observed order, lowest first.
#[test]
fn t1590_probed_characters() {
    assert_increasing(&[
        "", "\u{1}", "\u{1f}", "\u{7f}", "–", "—", "\u{a0}", "\t", "\n", "\r", "!", "~", "¡", "¦",
        "¨", "¯", "´", "¸", "¿", "‘", "’", "“", "”", "¢", "£", "¤", "¥", "€", "+", "±", "«", "»",
        "×", "÷", "§", "©", "¬", "®", "°", "µ", "…", "•", "0", "¼", "½", "¾", "¹", "²", "³", "9",
        "a", "ª", "á", "ä", "ã", "å", "a’b", "ab", "a'b", "a-b", "a–b", "a—b", "c", "ç", "d", "đ",
        "ð", "i", "ı", "ij", "l", "ł", "n", "ñ", "o", "º", "ö", "ø", "œ", "s", "t", "þ", "u", "z",
        "α", "β", "ω",
    ]);
    for (left, right) in [("ĳ", "ij"), ("Ĳ", "ij"), ("Z", "z")] {
        assert_eq!(
            cmp_text_excel(left, right),
            Ordering::Equal,
            "{left:?} = {right:?}"
        );
    }
}

#[test]
fn ascii_fast_path_agrees_with_the_general_path() {
    // A shared non-ASCII first character forces the general path without changing any pass:
    // equal letter and accent weights, and every hyphen/apostrophe position shifts by one.
    let samples = [
        "", "'", "-", " ", "!", "+", "0", "9", "a", "A", "a-b", "a'b", "ab", "co-op", "coop", "~",
        "-1", "'1", "1", "z", "Z_", "a\u{1}", "\u{7f}", "\t", "a\r",
    ];
    for left in samples {
        for right in samples {
            let prefixed = |s: &str| format!("\u{3b1}{s}");
            assert_eq!(
                cmp_text_excel(left, right),
                cmp_text_excel(&prefixed(left), &prefixed(right)),
                "{left:?} vs {right:?}"
            );
        }
    }
}

/// Two texts have the same collation key exactly when `cmp_text` finds them equal: the
/// exact-match index of XMATCH and XLOOKUP hashes the key (T-0448). Case, accents, expansions,
/// skipped hyphens and apostrophes, spaces, ASCII against non-ASCII, and lone surrogates.
#[test]
fn collation_key_equality_is_cmp_text_equality() {
    let mut texts: Vec<Text> = [
        "",
        "a",
        "A",
        "ab",
        "AB",
        "aB",
        "a-b",
        "a'b",
        "a\u{2013}b",
        "ab-",
        "-ab",
        "co-op",
        "coop",
        "Co-Op",
        "cafe",
        "café",
        "CAFÉ",
        "cafè",
        "ae",
        "æ",
        "Æ",
        "ss",
        "ß",
        "SS",
        "ij",
        "ĳ",
        "oe",
        "œ",
        "th",
        "þ",
        "u",
        "ü",
        "Ü",
        "ø",
        "ö",
        "d",
        "đ",
        "ð",
        "1",
        "¹",
        "01",
        " a",
        "a ",
        "a\u{a0}",
        "\u{1}a",
        "a\u{1}",
        "😀",
        "😀a",
        "x",
        "X",
        "ča",
        "Ča",
        "c\u{30c}a",
        "C\u{30c}-a",
        "č-a",
        "ca",
        "á",
        "a\u{301}",
        "ϊ",
        "ΐ",
        "ϊ\u{301}",
        "σ",
        "ς",
        "Σ",
        "е",
        "ё",
        "a\u{301}\u{30c}",
        "\u{301}a",
        "ª",
        "ª\u{301}",
        "à",
        "ǎ",
        "a\u{30c}",
        "a\u{342}\u{342}",
        "a\u{327}",
    ]
    .iter()
    .map(|s| Text::from(*s))
    .collect();
    texts.push(Text::from_utf16(vec![0xD800]));
    texts.push(Text::from_utf16(vec![0xD800, 0x61]));
    texts.push(Text::from_utf16(vec![0x61, 0xDC00]));
    for a in &texts {
        for b in &texts {
            assert_eq!(
                cmp_text(a, b) == Ordering::Equal,
                collation_key(a) == collation_key(b),
                "{a:?} vs {b:?}"
            );
        }
    }
}

/// Latin Extended-A, Greek and Cyrillic letters are their base letter plus an accent, and a
/// combining mark is the accent of the letter before it (probes/t3114-collation-beyond-latin1).
#[test]
fn letters_beyond_latin1_and_combining_marks() {
    assert_increasing(&["c", "ć", "č", "ca", "ča", "cb", "čb", "cz", "d"]);
    assert_increasing(&["a", "á", "ă", "ā", "ą", "b"]);
    assert_increasing(&["e", "é", "ė", "ě", "ē", "ę", "ea", "ěa", "eb"]);
    assert_increasing(&["o", "ö", "ő", "oa", "őa", "ob", "p"]);
    assert_increasing(&["u", "ū", "ů", "ų", "ű", "v", "z", "ź", "ż", "ž", "α"]);
    assert_increasing(&[
        "ι", "ί", "ϊ", "ΐ", "κ", "ω", "ώ", "а", "е", "ё", "ж", "и", "й", "к", "я",
    ]);
    for (a, b) in [
        ("a\u{301}", "á"),
        ("c\u{30c}", "č"),
        ("C\u{30c}", "č"),
        ("c\u{30c}a", "Ča"),
        ("Ő", "o\u{30b}"),
        ("Σ", "ς"),
        ("σ", "ς"),
        ("Ё", "ё"),
        ("Ά", "ά"),
        ("ϊ\u{301}", "ΐ"),
        // Pins an unprobed engine choice: a mark before a hyphen still belongs to its letter.
        ("C\u{30c}-a", "č-a"),
    ] {
        assert_eq!(cmp_text_excel(a, b), Ordering::Equal, "{a:?} = {b:?}");
    }
}

/// Accents are the Windows table's diacritic weights, and combining marks add theirs to the letter
/// before them in one wrapping byte (probes/t3154-collation-accents,
/// probes/t3154-collation-accent-weights, probes/t3154-collation-accent-pairs).
#[test]
fn accent_weights_add_up_and_wrap() {
    assert_increasing(&[
        "a", "á", "à", "â", "ä", "ǎ", "ă", "ā", "ã", "å", "ą", "ạ", "aa",
    ]);
    assert_increasing(&["e", "é", "è", "ė", "ê", "ë", "ě", "ē", "ę", "f"]);
    assert_increasing(&[
        "o", "ó", "ò", "ô", "ö", "ō", "õ", "ő", "ơ", "oa", "ơa", "ob",
    ]);
    assert_increasing(&["u", "ú", "ù", "û", "ü", "ū", "ů", "ų", "ű", "ư", "v"]);
    assert_increasing(&["s", "š", "ş", "ș", "sa", "șa", "sb"]);
    // Three dot-below marks wrap (2 + 3 * 86 = 260, so 4): between "a" and "á".
    assert_increasing(&["a", "a\u{323}\u{323}\u{323}", "á"]);
    assert_increasing(&["ª", "à", "ª\u{301}", "ȧ"]);
    for (a, b) in [
        ("a\u{301}\u{301}", "å"),
        ("a\u{301}\u{30c}", "a\u{30c}\u{301}"),
        ("á\u{30c}", "ǎ\u{301}"),
        ("à\u{323}", "a\u{323}\u{300}"),
        ("a\u{301}\u{300}", "ą"),
        ("a\u{342}\u{342}", "a\u{327}"),
        ("ǎ", "a\u{30c}"),
        ("Ș", "s\u{326}"),
        ("ț", "t\u{326}"),
        ("ư", "u\u{31b}"),
        ("ạ", "a\u{323}"),
        ("й", "и\u{306}"),
    ] {
        assert_eq!(cmp_text_excel(a, b), Ordering::Equal, "{a:?} = {b:?}");
        assert_eq!(collation_key(&Text::from(a)), collation_key(&Text::from(b)));
    }
    assert_ne!(cmp_text_excel("ș", "ş"), Ordering::Equal);
}

/// `decompose` binary-searches the table, so it must stay sorted by letter.
#[test]
fn decomposed_table_is_sorted() {
    use crate::functions::util::collation_tables::DECOMPOSED;
    assert!(DECOMPOSED.windows(2).all(|pair| pair[0].0 < pair[1].0));
}

/// The exact and wildcard matches fold `ς` to `σ` on both sides
/// (probes/t3114-collation-beyond-latin1 Calc!H10; Rust's final-sigma rule lower-cases a word-final
/// `Σ` to `ς`).
#[test]
fn final_sigma_folds_in_exact_and_wildcard_matches() {
    use crate::functions::criteria::{folded_units, WildcardPattern};
    for pattern in ["ς", "σ", "Σ"] {
        let compiled = WildcardPattern::new(&Text::from(pattern));
        for text in ["ς", "σ", "Σ"] {
            assert!(
                compiled.matches(&Text::from(text)),
                "{pattern:?} matches {text:?}"
            );
        }
    }
    assert!(WildcardPattern::new(&Text::from("ς*")).matches(&Text::from("σx")));
    assert_eq!(
        folded_units(&Text::from("ΟΔΟΣ")),
        folded_units(&Text::from("οδοσ"))
    );
    assert_eq!(
        WildcardPattern::new(&Text::from("οδοσ")).literal_units(),
        Some(folded_units(&Text::from("ΟΔΟΣ")))
    );
}

/// U+0001 is `*` in a pattern that starts with it and holds no `*`, and in SEARCH always
/// (probes/t3921-match-u0001-wildcard).
#[test]
fn soh_reads_as_a_star() {
    use crate::functions::criteria::WildcardPattern;
    let matches = |pattern: &str, text: &str| {
        WildcardPattern::new(&Text::from(pattern)).matches(&Text::from(text))
    };
    assert!(matches("\u{1}", "zzz") && matches("\u{1}", ""));
    assert!(matches("\u{1}b", "ab") && !matches("\u{1}b", "abc"));
    assert!(matches("\u{1}b\u{1}", "abc"));
    assert!(matches("~\u{1}", "zzz"));
    assert!(
        !matches("a\u{1}", "abc") && matches("a\u{1}c", "a\u{1}c") && !matches("a\u{1}c", "abc")
    );
    assert!(!matches("\u{1}*", "abc") && !matches("*\u{1}", "abc"));
    assert_eq!(
        WildcardPattern::new(&Text::from("\u{1}")).literal_units(),
        None
    );
    let search = |pattern: &str| {
        WildcardPattern::for_search(pattern.encode_utf16())
            .find_from(&"xabcx".encode_utf16().collect::<Vec<_>>(), 0)
    };
    assert_eq!(search("a\u{1}"), Some(1));
    assert_eq!(search("\u{1}*"), Some(0));
    assert_eq!(search("q\u{1}"), None);
}

/// The evaluated `Data` strings of the probes case `stem` (each a formula such as
/// `="a"&REPT(UNICHAR(769),22)`).
fn probe_data_strings(stem: &str) -> Vec<Text> {
    let path = format!(
        "{}/../../eval/suites/probes/cases/{stem}.case.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let case: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sheet = case["sheets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|sheet| sheet["name"] == "Data")
        .unwrap();
    let cells = sheet["cells"].as_object().unwrap();
    let mut model = crate::test::util::new_empty_model();
    for (cell, formula) in cells {
        model._set(cell, formula.as_str().unwrap());
    }
    model.evaluate();
    cells
        .keys()
        .map(|cell| Text::from(model._get_text(cell)))
        .collect()
}

/// The accent strings of the T-3154 and T-3201 probes and the kana strings of T-3275, on which
/// Excel's own comparison is not transitive (coercion.md "Not a total order"): `cmp_text`, which GROUPBY's sort uses, stays a total
/// order with `CollationKey` equality as its equality, and texts `cmp_text_pairwise` finds equal
/// share a key (what the criteria index relies on).
#[test]
fn cmp_text_is_a_total_order_on_the_accent_probes() {
    use crate::functions::util::collation::cmp_text_pairwise;
    for stem in [
        "t3154-collation-accent-weights",
        "t3154-collation-accent-pairs",
        "t3201-collation-long-mark-runs",
        "t3201-collation-wrapped-run-pairs",
        "t3275-collation-width-kana",
        "t3275-collation-width-kana-pairs",
    ] {
        let mut texts = probe_data_strings(stem);
        assert!(texts.len() > 20, "{stem}");
        texts.sort_by(cmp_text);
        for (i, a) in texts.iter().enumerate() {
            // Sorted, each text is equal to a run of the ones after it and below all the rest.
            let mut below = false;
            for b in &texts[i + 1..] {
                let order = cmp_text(a, b);
                assert_eq!(cmp_text(b, a), order.reverse(), "{stem}: {a:?} vs {b:?}");
                assert_eq!(
                    order,
                    if below {
                        Ordering::Less
                    } else {
                        order.min(Ordering::Equal)
                    },
                    "{stem}: {a:?} vs {b:?}"
                );
                below = order.is_lt();
                assert_eq!(
                    order.is_eq(),
                    collation_key(a) == collation_key(b),
                    "{stem}: {a:?} vs {b:?}"
                );
                if cmp_text_pairwise(a, b).is_eq() {
                    assert!(order.is_eq(), "{stem}: {a:?} vs {b:?}");
                }
            }
        }
    }
}

/// Excel's pairwise comparison: a text is below any text it is a proper prefix of, even when the
/// wrapped sums are equal or say otherwise (probes/t3181-collation-prefix-callers Calc!A1:A9,
/// probes/t3154-collation-accent-pairs Calc!S19, T18, W23, X22, AB28, AC27).
#[test]
fn pairwise_prefix_rule() {
    use crate::functions::util::collation::cmp_text_pairwise;
    let cmp = |a: &str, b: &str| cmp_text_pairwise(&Text::from(a), &Text::from(b));
    let circumflexes = format!("a{}", "\u{302}".repeat(16));
    assert_eq!(cmp("a", &circumflexes), Ordering::Less);
    assert_eq!(cmp(&circumflexes, "a"), Ordering::Greater);
    assert_eq!(
        cmp_text(&Text::from("a"), &Text::from(circumflexes.as_str())),
        Ordering::Equal
    );
    // U+0342 (141) < two U+0342 (28, = cedilla) < U+0342.
    assert_eq!(cmp("a\u{342}", "a\u{342}\u{342}"), Ordering::Less);
    assert_eq!(cmp("a\u{342}\u{342}", "a\u{327}"), Ordering::Equal);
    assert_eq!(cmp("a\u{327}", "a\u{342}"), Ordering::Less);
    // U+0323 (86) < three U+0323 (4) < á (14) < U+0323.
    assert_eq!(cmp("a\u{323}", "a\u{323}\u{323}\u{323}"), Ordering::Less);
    assert_eq!(cmp("a\u{323}\u{323}\u{323}", "á"), Ordering::Less);
    assert_eq!(cmp("á", "a\u{323}"), Ordering::Less);
    // Precomposed á is one unit: it equals a plus U+0301.
    assert_eq!(cmp("á", "a\u{301}"), Ordering::Equal);
}

/// A mark after a hyphen or apostrophe goes with that skipped character
/// (probes/t3304-collation-multi-letter-accents G4; only U+0301 probed).
#[test]
fn mark_after_skipped_character_raises_it() {
    assert_increasing(&["a'b", "a-b", "a'\u{301}b", "a-\u{301}b", "\u{e1}b"]);
    // The mark adds nothing to the accent pass: "a-" + U+0301 + "b" is below "á-b".
    assert_eq!(cmp_text_excel("a-\u{301}b", "\u{e1}-b"), Ordering::Less);
    assert_eq!(cmp_text_excel("a-\u{301}b", "\u{e1}'b"), Ordering::Less);
}

/// Full-width forms, kana, voicing and the long-vowel and iteration marks
/// (probes/t3275-collation-width-kana, probes/t3275-collation-width-kana-pairs; coercion.md
/// "Full-width and half-width forms, kana, voicing and the long-vowel mark").
#[test]
fn width_and_kana_tiebreaks() {
    use crate::functions::util::collation::cmp_text_pairwise;
    assert_increasing(&["0", "０", "1", "１", "１a", "1b"]);
    assert_increasing(&[" ", "\u{3000}", "!", "！", "#"]);
    assert_increasing(&[
        "A", "Ａ", "á", "aa", "Ａa", "aá", "ab", "a＇b", "a-b", "a－b", "AＢ", "Ａb",
    ]);
    assert_increasing(&["", "'", "＇", "-", "－", "゛", " "]);
    assert_increasing(&[
        "я", "ぁ", "ｱ", "ア", "あ", "ｱあ", "アあ", "あア", "ああ", "ぁい", "い",
    ]);
    assert_increasing(&["かぁ", "カア", "かあ", "カー", "かー", "があ", "かい"]);
    assert_increasing(&["か", "ｶﾞ", "ガ", "が", "ッ", "っ", "つ", "は", "ば", "ぱ"]);
    assert_increasing(&["かか", "かゝ", "かが", "かゞ", "き", "きい", "きー"]);
    assert_increasing(&["一", "人", "人々", "人人", "ー", "ーa"]);
    for (left, right) in [
        ("Ａ", "ａ"),
        ("か\u{3099}", "が"),
        ("は\u{309a}", "ぱ"),
        ("ー", "ゝ"),
        ("ゞ", "々"),
    ] {
        assert_eq!(
            cmp_text_excel(left, right),
            Ordering::Equal,
            "{left:?} = {right:?}"
        );
    }
    // `"aー"` sorts as `"aa"`, but Excel's `<` compares its `ー` alone where the texts differ.
    let cmp = |a: &str, b: &str| cmp_text_pairwise(&Text::from(a), &Text::from(b));
    assert_eq!(cmp_text_excel("aー", "aa"), Ordering::Equal);
    assert_eq!(cmp("aー", "aa"), Ordering::Greater);
    assert_eq!(cmp("aー", "ab"), Ordering::Greater);
    assert_eq!(cmp("aー", "AＢ"), Ordering::Greater);
    assert_eq!(cmp("aー", "Ａa"), Ordering::Less);
    assert_eq!(cmp("aー", "Ａb"), Ordering::Less);
}

/// A long-vowel or iteration mark with no letter before it sorts above every other character,
/// whatever follows it (probes/t4258-collation-lone-mark-astral; t3465-collation-holdout C313:D313).
#[test]
fn lone_mark_sorts_above_every_character() {
    let cmp = |a: &str, b: &str| cmp_text_pairwise(&Text::from(a), &Text::from(b));
    for mark in ["ー", "ゝ", "ゞ", "ヽ", "ヾ", "々"] {
        for other in [
            "𠀀",
            "𪜀",
            "\u{2fa1d}",
            "𰀀",
            "가",
            "一",
            "\u{9fff}",
            "㐀",
            "豈",
            "\u{e000}",
            "😀",
            "\u{fffd}",
            "\u{f0000}",
            "\u{10fffd}",
            "𝐀",
            "𠀀𠀀",
        ] {
            assert_eq!(cmp(mark, other), Ordering::Greater, "{mark:?} > {other:?}");
            assert_eq!(cmp(other, mark), Ordering::Less, "{other:?} < {mark:?}");
        }
    }
    for text in [
        "々\u{35f}",
        "々ヲ",
        "々一",
        "々𠀀",
        "々A",
        "々ー",
        "ーヲ",
        "ー𠀀",
    ] {
        assert_eq!(cmp(text, "𠀀"), Ordering::Greater, "{text:?} > 𠀀");
    }
    let b313 = "々\u{35f}ヲ\u{32b}\u{350}ゞ";
    assert_eq!(cmp("𠀀", b313), Ordering::Less);
    assert_eq!(cmp(b313, "𠀀"), Ordering::Greater);
}

/// Exact MATCH folds `ー` after a letter to that letter and a leading iteration mark to `ー`
/// (probes/t3275-collation-width-kana Calc!C127:C129, C136), and does not decompose voicing.
#[test]
fn exact_match_fold_of_the_marks() {
    use crate::functions::util::collation::fold_exact_lookup;
    let fold = |text: &str| {
        let units: Vec<u16> = text.encode_utf16().collect();
        String::from_utf16_lossy(&fold_exact_lookup(&units))
    };
    assert_eq!(fold("aー"), "aa");
    assert_eq!(fold("ゝ"), "ー");
    assert_eq!(fold("々"), "ー");
    assert_eq!(fold("かー"), "かー");
    assert_eq!(fold("かゝ"), "かゝ");
    assert_eq!(fold("a々"), "a々");
    assert_ne!(fold("か\u{3099}"), fold("が"));
    assert_eq!(fold_exact_lookup(&[0xD800, 0x30FC]), vec![0xD800, 0x30FC]);
}

/// A cycle `a<b<c<d<a` with chords `c<a`, `d<b` (probes/t3418-sort-cycle-permutations, the shape of
/// Data!A3, A4, A6, A7): Excel does not reverse the strictly descending chain `a, c, b, d`; it merges
/// one-element runs pairwise, `c, a` and `d, b`, then `c, d, a, b` (Calc!A3, T-3418).
#[test]
fn the_pairwise_sort_never_reverses_a_descending_chain() {
    use crate::functions::util::sort_indices_pairwise;
    use std::cmp::Ordering;
    // less[x][y]: x < y. a=0, b=1, c=2, d=3.
    let less = [
        [false, true, false, false],
        [false, false, true, false],
        [true, false, false, true],
        [true, true, false, false],
    ];
    let cmp = |x: usize, y: usize| {
        if less[x][y] {
            Ordering::Less
        } else if less[y][x] {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    };
    let mut indices = vec![0, 2, 1, 3];
    sort_indices_pairwise(&mut indices, cmp);
    assert_eq!(indices, vec![2, 3, 0, 1]);
}

/// UNIQUE's grouping (`coalesce_equal_lines`) on the cycle `c < a < b = c` of Data!A5, A3, A4 of
/// probes/t4200-unique-cycle-permutations (a=A3, b=A4, c=A5): equal neighbours become one group
/// before a merge can put `a` between them, so `{a;c;b}` keeps a and c (row 146) though SORT's
/// order is c, a, b; `{b;a;c}` keeps all three (row 147); on a total order it is equality (T-4200).
#[test]
fn unique_grouping_coalesces_equal_lines_before_merging() {
    use crate::functions::util::coalesce_equal_lines;
    use std::cmp::Ordering;
    // a=0, b=1, c=2: a < b, c < a, b = c.
    let cmp = |x: usize, y: usize| -> Ordering {
        match (x, y) {
            (0, 1) | (2, 0) => Ordering::Less,
            (1, 0) | (0, 2) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    };
    let group = |order: &[usize]| {
        coalesce_equal_lines::<()>(order.len(), |i, j| Ok(cmp(order[i], order[j])))
            .unwrap_or_default()
    };
    assert_eq!(group(&[0, 2, 1]), vec![0, 1, 1]);
    assert_eq!(group(&[2, 1, 0]), vec![0, 0, 2]);
    assert_eq!(group(&[1, 0, 2]), vec![0, 1, 2]);
    assert_eq!(group(&[0, 1, 2]), vec![0, 1, 1]);
    // A total order: groups by equality, each line pointing at the first of its value.
    let values = [3, 1, 3, 2, 1, 3];
    let first = coalesce_equal_lines::<()>(values.len(), |i, j| Ok(values[i].cmp(&values[j])));
    assert_eq!(first, Ok(vec![0, 1, 0, 3, 1, 0]));
    // An error from the comparison ends the walk.
    assert_eq!(coalesce_equal_lines(3, |_, _| Err("budget")), Err("budget"));
}

/// UNIQUE's merge on two-field rows of A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C, B = C, A < B;
/// x < y), compared field by field (T-4393, probes/t4380-unique-keep-head-holdout rows 6 and 10):
/// an equal right head joins the left head, which stays, and nothing out of a merge is compared
/// with the group out before it. Over Cy Cx Ay Bx Ay the second Ay never meets Cy and is kept.
#[test]
fn unique_merge_keeps_the_left_head_and_never_coalesces_its_output() {
    use crate::functions::util::coalesce_equal_lines;
    use std::cmp::Ordering;
    let text = |a: u8, b: u8| -> Ordering {
        match (a, b) {
            _ if a == b => Ordering::Equal,
            (b'A', b'B') => Ordering::Less,
            (b'B', b'A') => Ordering::Greater,
            _ => Ordering::Equal,
        }
    };
    let group = |run: &str| {
        let rows: Vec<&[u8]> = run.split(' ').map(str::as_bytes).collect();
        coalesce_equal_lines::<()>(rows.len(), |i, j| {
            Ok(text(rows[i][0], rows[j][0]).then(rows[i][1].cmp(&rows[j][1])))
        })
        .unwrap_or_default()
    };
    // Kept: Cy, Cx, Bx, Ay (row 3's Ay joins Cy; row 5's stays apart).
    assert_eq!(group("Cy Cx Ay Bx Ay"), vec![0, 1, 0, 3, 4]);
    // Kept: Cy, Cx, Bx (the last Cy joins Cy).
    assert_eq!(group("Cy Cx Ay Bx Cy"), vec![0, 1, 0, 3, 0]);
}

/// SORT's sort (`sort_indices_pairwise`) under Excel's intransitive text comparison
/// (`cmp_text_pairwise`) ends on the data of probes/t3201-collation-wrapped-run-pairs (37 texts,
/// hundreds of intransitive triples) in several input orders, without panicking, with a
/// permutation of the input in which no text is directly after one it is below (T-3211).
#[test]
fn the_pairwise_sort_ends_on_intransitive_texts() {
    use crate::functions::util::collation::cmp_text_pairwise;
    use crate::functions::util::sort_indices_pairwise;
    let texts = probe_data_strings("t3201-collation-wrapped-run-pairs");
    assert!(texts.len() > 30);
    let n = texts.len();
    // The identity, its reverse, and interleavings that cut the runs differently.
    let orders: Vec<Vec<usize>> = vec![
        (0..n).collect(),
        (0..n).rev().collect(),
        (0..n).step_by(2).chain((1..n).step_by(2)).collect(),
        (0..n).map(|i| (i * 7) % n).collect(),
        (0..n).map(|i| (i * 11 + 5) % n).collect(),
    ];
    for order in orders {
        for descending in [false, true] {
            let mut indices = order.clone();
            sort_indices_pairwise(&mut indices, |a, b| {
                let ordering = cmp_text_pairwise(&texts[a], &texts[b]);
                if descending {
                    ordering.reverse()
                } else {
                    ordering
                }
            });
            let mut seen = indices.clone();
            seen.sort_unstable();
            assert_eq!(seen, (0..n).collect::<Vec<_>>(), "not a permutation");
            for pair in indices.windows(2) {
                let ordering = cmp_text_pairwise(&texts[pair[1]], &texts[pair[0]]);
                let ordering = if descending {
                    ordering.reverse()
                } else {
                    ordering
                };
                assert_ne!(
                    ordering,
                    Ordering::Less,
                    "{:?} after {:?}",
                    texts[pair[1]],
                    texts[pair[0]]
                );
            }
        }
    }
}

/// probes/t3276-collation-other-scripts (T-3276, excel-graph; T-3324): block order, marks as
/// accents, Thai prevowels, other-script digits and the symbols above `€`.
#[test]
fn t3276_other_scripts() {
    // Block order (ranks A18:A21, A44, A79, A95, A96; G2:G8, G35).
    assert_increasing(&[
        "ω", "я", "ア", "あ", "א", "ת", "ا", "ي", "क", "ह", "ก", "ฮ", "ກ", "ກເ", "ຂ", "ເກ", "𝐀",
        "𝐀b", "😀", "가", "힣", "一", "龍", "𠀀",
    ]);
    // Marks are accents (G13:G16, G19, G20); Devanagari matras and virama are letters (G17, G18).
    assert_increasing(&["ב", "ב\u{5b7}", "ב\u{5bc}", "בא", "ב\u{5bc}א", "בב"]);
    assert_increasing(&["ب", "ب\u{64e}", "ب\u{651}", "با", "ب\u{64e}ا", "بب"]);
    assert_increasing(&["ก", "ก\u{e48}", "กก", "ก\u{e48}ก", "กข"]);
    assert_increasing(&["क", "कख", "का", "कि", "क्", "ख"]);
    // Thai prevowels before the consonants, Thai `า` after them (G21:G23).
    assert_increasing(&["เก", "เข", "แก", "ก", "กเ", "กก", "กข", "กา", "ข"]);
    // Other-script digits are their digit plus a tiebreak (G25:G27).
    assert_increasing(&["3", "٣", "३", "3a", "٣a", "३a", "3b"]);
    assert_increasing(&["2", "٢", "3", "4", "٤", "9"]);
    // Symbols above `€` (G29:G31; ranks A89, A91).
    assert_increasing(&[
        "~", "€", "+", ">", "←", "☺", "♥", "✓", "0", "9", "∞", "a", "Ab", "™", "ÿ",
    ]);
    assert_ne!(cmp_text_excel("™", "tm"), Ordering::Equal);
    // A lone surrogate stays above U+FFFD and brackets the pair it belongs to
    // (probes/text-lone-surrogates A18:A24).
    let smile: Vec<u16> = "😀".encode_utf16().collect();
    let high = Text::from_utf16(smile[..1].to_vec());
    let low = Text::from_utf16(smile[1..].to_vec());
    let fffd = Text::from("\u{fffd}");
    let pair = Text::from("😀");
    assert_eq!(cmp_text(&high, &pair), Ordering::Less);
    assert_eq!(cmp_text(&pair, &low), Ordering::Less);
    assert_eq!(cmp_text(&high, &fffd), Ordering::Greater);
    assert_eq!(cmp_text(&low, &fffd), Ordering::Greater);
}

/// T-3917 rule 7: every other-script digit is its digit plus a tiebreak in code-point order of
/// its block, below the next digit (probes/t3559-collation-unprobed-blocks ranks Calc!A11, A136:A145,
/// G21:G25; probes/t3895-collation-script-digits A151:A178).
#[test]
fn script_digits_tiebreak_by_block() {
    assert_increasing(&[
        "3", "３", "٣", "۳", "३", "৩", "੩", "௩", "౩", "๓", "໓", "༣", "၃", "៣", "3a", "4",
    ]);
    for digit in ["੩", "௩", "౩", "៣"] {
        assert_ne!(cmp_text_excel(digit, "3"), Ordering::Equal);
    }
}

/// T-3917 rule 1: `➿`, `ᡸ` and `₿` are equal to each other, above `""` and below `"!"`
/// (probes/t3559-collation-unprobed-blocks ranks A35, A62, A126, B35, G17). They are weightless
/// (T-3904): Excel's `<` puts them above `""` by the prefix rule, while the total order and the
/// key, which weigh them nothing, tie them with it.
#[test]
fn t3559_unknown_characters_weigh_alike() {
    for unknown in ["\u{27bf}", "\u{1878}", "\u{20bf}"] {
        let pairwise = |a: &str, b: &str| cmp_text_pairwise(&Text::from(a), &Text::from(b));
        assert_eq!(pairwise("", unknown), Ordering::Less);
        assert_eq!(cmp_text_excel("", unknown), Ordering::Equal);
        assert_increasing(&[unknown, "!", "0"]);
        assert_eq!(cmp_text_excel(unknown, "\u{27bf}"), Ordering::Equal);
    }
}

/// T-3917 rule 2: symbols among the punctuation in Excel's order, `•` after them (ranks 4 to 29,
/// G16, G18).
#[test]
fn t3559_symbol_order() {
    assert_increasing(&[
        "!", "#", "$", "%", "、", "。", "~", "「", "₩", "€", "₹", "+", "≠", "∑", "∫", "≤", "■",
        "●", "←", "⇿", "─", "█", "☀", "✓", "•", "0",
    ]);
}

/// T-3917 rule 3: `∞` between `"9"` and `"a"`, `℃ №` between `"a"` and `"tm"`, `™` after `"tn"`
/// and before `"z"` (ranks 41 to 50, G1:G12).
#[test]
fn t3559_letterlike_symbols() {
    assert_increasing(&["~", "•", "0", "9", "∞", "a", "℃", "№", "tm", "™", "z"]);
    assert_increasing(&["t", "tn", "™"]);
    assert_eq!(cmp_text_excel("tm", "TM"), Ordering::Equal);
}

/// T-3917 rules 4 and 5: block order and the exceptions to code-point order inside a block
/// (ranks 51 to 154).
#[test]
fn t3559_block_order() {
    assert_increasing(&[
        "ω", "я", "ա", "ֆ", "ა", "ჰ", "Ⴀ", "あ", "א", "ي", "क", "অ", "ক", "কি", "হ", "ਅ", "ਹ", "ਕ",
        "ਕਿ", "அ", "க", "ஹ", "అ", "క", "హ", "เก", "โ", "ไก", "ก", "ข", "ะ", "ำ", "က", "ကိ", "အ",
        "ᠠ", "ཀ", "ཀི", "ཨ", "ກ", "ກະ", "ຂ", "ະ", "ແ", "ໂ", "ໄ", "ໃ", "ក", "កា", "អ", "ሀ", "ፚ", "𐀀",
        "𝐀", "😀", "𰀀", "가", "一", "𠀀", "𪜀",
    ]);
    assert_ne!(cmp_text_excel("Ⴀ", "ა"), Ordering::Equal);
}

/// T-3917 rule 6: equal forms (G13:G15, B55, B97, B109).
#[test]
fn t3559_equal_forms() {
    assert_eq!(cmp_text_excel("Ա", "ա"), Ordering::Equal);
    assert_eq!(cmp_text_excel("ץ", "צ"), Ordering::Equal);
    assert_eq!(cmp_text_excel("\u{fefb}", "لا"), Ordering::Equal);
    assert_increasing(&["צ", "ש", "ת"]);
}

/// T-4013: `=` equates the members of Excel's wider classes and keeps supplementary case pairs
/// apart (probes/t3584-match-fold-weightless Calc!B21:F21, B25:F25, P7, P9;
/// probes/t3929-wildcard-fold-criteria rows 29-30).
#[test]
fn t4013_classes_are_equal_and_astral_case_pairs_apart() {
    let equal = |a: &str, b: &str| {
        let (a, b) = (Text::from(a), Text::from(b));
        cmp_text_pairwise(&a, &b) == Ordering::Equal && collation_key(&a) == collation_key(&b)
    };
    for (a, b) in [
        ("\u{1f05}", "\u{1f0a}"), // Greek Extended: dasia+oxia with psili+varia
        ("x\u{1f0d}y", "X\u{1f02}Y"),
        ("\u{3f5}", "\u{3b5}"),  // ϵ with ε
        ("\u{1fbe}", "\u{3b9}"), // prosgegrammeni with ι
        ("\u{1fbe}", "\u{399}"),
        ("\u{376}", "\u{3f3}"), // Ͷ with ϳ
        ("\u{3d1}", "\u{3f4}"), // ϑ with ϴ
    ] {
        assert!(equal(a, b), "{a:?} = {b:?}");
    }
    assert!(!equal("\u{10400}", "\u{10428}"), "Deseret keeps its case");
    // ϴ is not θ: probes/t4013-class-equality-order Calc!B13 FALSE, D13 TRUE.
    assert!(!equal("\u{3f4}", "\u{3b8}"), "ϴ is not θ");
    // Class members weigh alike, so the character after them decides the order.
    let (a, b) = (Text::from("\u{3d1}a"), Text::from("\u{3f4}b"));
    assert_eq!(cmp_text(&a, &b), Ordering::Less);
    assert_eq!(cmp_text(&b, &a), Ordering::Greater);
}

/// T-3904: weightless characters weigh nothing, except in the tail one text has beyond the
/// other after a shared start of units below U+007B (coercion.md "Format characters, U+200B and
/// the C1 controls (T-3662)"; probes/t3662-collation-ignorables, -prefix, -sweep).
#[test]
fn t3904_weightless_characters_and_the_prefix_rule() {
    let pairwise = |a: &str, b: &str| cmp_text_pairwise(&Text::from(a), &Text::from(b));
    let same_key =
        |a: &str, b: &str| collation_key(&Text::from(a)) == collation_key(&Text::from(b));
    for c in ['\u{ad}', '\u{200d}', '\u{feff}', '\u{524}', '\u{e83}'] {
        let equal = [
            (format!("a{c}b"), "ab".to_owned()),
            (format!("{c}ab"), "ab".to_owned()),
            (format!("é{c}"), "é".to_owned()),
            (format!("a~{c}"), "a~".to_owned()),
            (format!("ab{c}"), format!("ab{c}{c}")),
            (c.to_string(), "\u{525}".to_owned()),
        ];
        for (a, b) in &equal {
            assert_eq!(pairwise(a, b), Ordering::Equal, "{a:?} = {b:?}");
            assert!(same_key(a, b), "{a:?} and {b:?} share a key");
        }
        // The prefix rule: a shorter fast start is less, case aside, though the keys agree.
        for (a, b) in [
            ("ab".to_owned(), format!("ab{c}")),
            ("AB".to_owned(), format!("ab{c}")),
            (String::new(), c.to_string()),
            ("a-".to_owned(), format!("a-{c}")),
        ] {
            assert_eq!(pairwise(&a, &b), Ordering::Less, "{a:?} < {b:?}");
            assert_eq!(pairwise(&b, &a), Ordering::Greater, "{b:?} > {a:?}");
            assert!(same_key(&a, &b), "{a:?} and {b:?} share a key");
        }
        // A difference goes to the collation: "ab"&c < "ac" and < "b" (-sweep F1, F5).
        assert_eq!(pairwise(&format!("ab{c}"), "ac"), Ordering::Less);
        assert_eq!(pairwise(&format!("ab{c}"), "b"), Ordering::Less);
    }
}

/// T-4039: the Unicode spaces, U+200B and the C1 controls (probes/t4039-collation-space-forms).
#[test]
fn t4039_space_forms_zero_width_space_and_c1_controls() {
    assert_increasing(&[
        " ", "\u{3000}", "\u{202f}", "\u{a0}", "\t", "\u{2002}", "\u{2003}", "\u{2004}",
        "\u{200a}", "\u{200b}", "!", "\u{ac}", "\u{2022}", "\u{80}", "\u{81}", "\u{9f}", "0",
    ]);
    assert_eq!(cmp_text_excel("\u{205f}", " "), Ordering::Equal);
    assert_eq!(cmp_text_excel("\u{2000}", "\u{2002}"), Ordering::Equal);
    assert_eq!(cmp_text_excel("\u{2001}", "\u{2003}"), Ordering::Equal);
    assert_eq!(cmp_text_excel("a\u{200b}b", "ab"), Ordering::Less);
}

/// T-4072: U+202F is the space at the first pass, after U+3000; a mark after a weightless
/// character is a lone mark (probes/t4060-collation-open-choices Calc!B2, K2, B50:B63).
#[test]
fn t4072_narrow_nbsp_and_marks_after_weightless_characters() {
    let pairwise = |a: &str, b: &str| cmp_text_pairwise(&Text::from(a), &Text::from(b));
    assert_eq!(pairwise("\u{202f}a", " b"), Ordering::Less);
    assert_increasing(&[" ", "\u{3000}", "\u{202f}", "\u{a0}"]);
    for weightless in ['\u{ad}', '\u{524}'] {
        let marked = format!("a{weightless}\u{301}");
        assert_eq!(pairwise(&marked, "a"), Ordering::Greater, "{marked:?} > a");
        assert_eq!(
            pairwise(&marked, "\u{e1}"),
            Ordering::Less,
            "{marked:?} < á"
        );
        assert_eq!(pairwise(&format!("{marked}b"), "\u{e1}b"), Ordering::Less);
        assert_eq!(pairwise(&format!("{marked}b"), "ab"), Ordering::Greater);
        assert_eq!(
            pairwise(&format!("{weightless}\u{301}"), "\u{301}"),
            Ordering::Equal
        );
        // Without a mark after it, a weightless character still weighs nothing.
        assert_eq!(pairwise(&format!("a{weightless}b"), "ab"), Ordering::Equal);
    }
}

/// T-4087: U+202F differs from the space at the accent pass; a weightless character keeps a
/// hyphen's marks; width and superscripts are weighed before small kana, kana type after
/// (probes/t4086-collation-later-passes).
#[test]
fn t4087_later_passes() {
    let pairwise = |a: &str, b: &str| cmp_text_pairwise(&Text::from(a), &Text::from(b));
    // U+202F against an accent, either position.
    assert_eq!(pairwise("\u{202f}a", " \u{e1}"), Ordering::Greater);
    assert_eq!(pairwise("a\u{202f}", "\u{e1} "), Ordering::Less);
    assert_increasing(&[" ", "\u{3000}", "\u{202f}", "\u{a0}"]);
    // A hyphen or apostrophe keeps its mark across a weightless character.
    assert_eq!(pairwise("a-\u{ad}\u{301}", "a-\u{301}"), Ordering::Equal);
    assert_eq!(pairwise("a'\u{ad}\u{301}b", "a'\u{301}b"), Ordering::Equal);
    assert_eq!(pairwise("a-\u{ad}\u{301}", "a-"), Ordering::Greater);
    // Width and superscripts before small kana, from either position; kana type after it.
    assert_eq!(pairwise("\u{ff21}\u{3041}", "A\u{3042}"), Ordering::Greater);
    assert_eq!(pairwise("\u{3041}\u{ff21}", "\u{3042}A"), Ordering::Greater);
    assert_eq!(pairwise("\u{b9}\u{3041}", "1\u{3042}"), Ordering::Greater);
    assert_eq!(pairwise("\u{3041}\u{3000}", "\u{3042} "), Ordering::Greater);
    assert_eq!(
        pairwise("\u{30a2}\u{3041}", "\u{3042}\u{3042}"),
        Ordering::Less
    );
}

/// T-4153: U+202F's accent weight is 10: above `" "&U+030D` (5) and two U+030D (8), below three
/// (11) and `" "&U+0301` (14), equal to the space with 88 U+030D (266 wraps to 10)
/// (probes/t4153-narrow-nbsp-accent Calc!B1:D2, B30:D31, C34).
#[test]
fn t4153_narrow_nbsp_accent_weight() {
    let pairwise = |a: &str, b: &str| cmp_text_pairwise(&Text::from(a), &Text::from(b));
    let marks = |n: usize| format!(" {}", "\u{30d}".repeat(n));
    assert_eq!(pairwise("\u{202f}", &marks(1)), Ordering::Greater);
    assert_eq!(pairwise("\u{202f}", &marks(2)), Ordering::Greater);
    assert_eq!(pairwise("\u{202f}", &marks(3)), Ordering::Less);
    assert_eq!(pairwise("\u{202f}", " \u{301}"), Ordering::Less);
    assert_eq!(pairwise("\u{202f}", &marks(88)), Ordering::Equal);
}

/// T-4234: on all-ASCII texts Excel's pairwise order agrees with the total `cmp_text`, the
/// invariant behind UNIQUE's hash path (`unique_of` keeps the hash buckets when every text is
/// ASCII and uses `coalesce_equal_lines` otherwise; T-4200 review).
pub(crate) fn ascii_texts() -> Vec<Text> {
    // Prefixes, case, hyphen, apostrophe, space, digits, punctuation and controls.
    let alphabet = [
        'a', 'A', 'b', 'B', 'z', '0', '9', '-', '\'', ' ', '_', '.', '~', '\t', '\n', '\u{1}',
        '\u{7f}',
    ];
    let mut texts = vec![String::new()];
    for a in alphabet {
        texts.push(a.to_string());
        for b in alphabet {
            texts.push(format!("{a}{b}"));
        }
    }
    for prefix in ["ab", "Ab", "co-op", "coop", "it's", "its", "a b", "ab "] {
        for suffix in ["", "-", "'", "a", "A", " ", "1"] {
            texts.push(format!("{prefix}{suffix}"));
        }
    }
    texts.iter().map(|s| Text::from(s.as_str())).collect()
}

#[test]
fn t4234_ascii_pairwise_agrees_with_total_order() {
    let texts = ascii_texts();
    for a in &texts {
        for b in &texts {
            assert_eq!(
                cmp_text_pairwise(a, b),
                cmp_text(a, b),
                "{:?} vs {:?}",
                a.to_string(),
                b.to_string()
            );
        }
    }
}

/// T-4270: the 32 C1 controls U+0080 to U+009F are pairwise distinct and sort by code point, after
/// U+200B, `¬` and `•` and before `0`, alone and between `a` and `b` (probes/t4270-collation-c1-controls
/// Calc!B1:B36 ranks 3 to 34 in row order, U+200B 0, `¬` 1, `•` 2, `0` 35; C1:C36 and E1:E36 all 1;
/// N1:N32 the code points of SORT in order).
#[test]
fn t4270_c1_controls_in_code_point_order() {
    let controls: Vec<String> = ('\u{80}'..='\u{9f}').map(String::from).collect();
    let mut alone = vec!["\u{200b}", "¬", "•"];
    alone.extend(controls.iter().map(String::as_str));
    alone.push("0");
    assert_increasing(&alone);
    let wrapped: Vec<String> = controls.iter().map(|c| format!("a{c}b")).collect();
    let mut wrapped: Vec<&str> = wrapped.iter().map(String::as_str).collect();
    wrapped.push("ab");
    assert_increasing(&wrapped);
}
