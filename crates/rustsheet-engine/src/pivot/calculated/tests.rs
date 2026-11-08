#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! Calculated fields against the probes' sidecars: each pivot is the case file's definition
//! (`pivot::probe_cases`, which appends the builder's calculated cache fields), and every
//! expectation is a cell Excel saved (probes/t4505-pivot-calc-fields,
//! t4505-pivot-calc-field-functions, t4130-pivot-calc-items CF; pivots.md "Calculated fields as
//! Excel for the web renders them").

use super::*;
use crate::pivot::probe_cases::{probe_pivot, read, sidecar_differences};
use crate::pivot::{compute, MAX_PIVOT_WORK};

fn check(stem: &str, sheets: &[&str]) {
    let case = read(stem, "case");
    let wrong: Vec<String> = sheets
        .iter()
        .flat_map(|name| {
            let (cache, table) = probe_pivot(&case, name);
            sidecar_differences(stem, name, &cache, &table, None, &[])
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn every_level_computes_the_formula_over_its_own_sums() {
    // LV: East Total 210 = 6×35, Grand Total 870 = 15×58; LC: the column grand total 360 = 9×40;
    // CNT: Count, Average, Max and Min show the Sum; DZ: errors stay where the sums make them;
    // NV: a text field is 0; K: constants; FA: functions over sums; NM: quoted names; CC: a
    // calculated field over another.
    check(
        "t4505-pivot-calc-fields",
        &["LV", "LC", "CNT", "DZ", "NV", "K", "FA", "NM", "CC"],
    );
}

#[test]
fn functions_see_each_field_as_its_sum_and_a_cycle_is_name() {
    // FX..FV: INDEX, ROWS, LOOKUP, MATCH, VLOOKUP over the sums; ET: a text constant is #VALUE!;
    // EC, ES: a cycle and a self-reference are #NAME?; EF: a field listed later.
    check(
        "t4505-pivot-calc-field-functions",
        &[
            "FX", "FR", "FL", "FK", "FM", "FN", "FV", "ET", "EC", "ES", "EF",
        ],
    );
}

#[test]
fn a_calculated_field_beside_calculated_items_off() {
    check("t4130-pivot-calc-items", &["CF"]);
}

fn field(name: &str, formula: Option<&str>) -> CacheField {
    CacheField {
        name: name.to_string(),
        formula: formula.map(str::to_string),
        ..CacheField::default()
    }
}

fn checked(formula: &str) -> Result<(), CalculatedFieldError> {
    let fields = [
        field("Units", None),
        field("Unit Cost", None),
        field("Rev2", None),
        field("F", Some(formula)),
    ];
    check_calculated_fields(&fields)
}

#[test]
fn the_formulas_excel_for_the_web_refuses_are_refused() {
    // pivots.md "Pivot flags Excel for the web refuses" (T-4505 runs).
    use CalculatedFieldError::{ArrayConstant, Function as Call, Reference, UnknownField};
    assert_eq!(checked("NOW()+Units"), Err(Call("NOW".into())));
    assert_eq!(checked("TODAY()"), Err(Call("TODAY".into())));
    assert_eq!(checked("RAND()*Units"), Err(Call("RAND".into())));
    assert_eq!(checked("INDIRECT(\"A1\")"), Err(Call("INDIRECT".into())));
    assert_eq!(checked("SUM({1,2})+Units"), Err(ArrayConstant));
    assert_eq!(checked("Data!C2*2"), Err(Reference));
    assert_eq!(checked("A1*2"), Err(Reference));
    assert_eq!(checked("Foo*2"), Err(UnknownField("Foo".into())));
    // Engine choices (UNVERIFIED, T-4897).
    assert_eq!(checked("Units:Units"), Err(Reference));
    assert!(checked("LET(x,1,x)").is_err());
    assert_eq!(checked("1+"), Err(CalculatedFieldError::Parse));
}

#[test]
fn fields_are_named_bare_quoted_or_spelled_like_a_cell() {
    assert_eq!(checked("Units*2"), Ok(()));
    assert_eq!(checked("units*2"), Ok(()));
    assert_eq!(checked("'Unit Cost'*2+IF(Units>3,1,0)"), Ok(()));
    assert_eq!(checked("Rev2+1"), Ok(()));
    assert_eq!(checked("F+1"), Ok(()));
    assert_eq!(checked("\"a\""), Ok(()));
}

#[test]
fn formulas_are_bounded_in_length_and_number() {
    let long = format!("Units{}", "+1".repeat(MAX_CALCULATED_FORMULA_CHARS / 2));
    assert_eq!(checked(&long), Err(CalculatedFieldError::TooLong));
    let mut fields = vec![field("Units", None)];
    for i in 0..=MAX_CALCULATED_FIELDS {
        fields.push(field(&format!("C{i}x"), Some("Units")));
    }
    assert_eq!(
        check_calculated_fields(&fields),
        Err(CalculatedFieldError::TooMany)
    );
}

#[test]
fn the_formulas_draw_on_the_pivots_work_budget() {
    let case = read("t4505-pivot-calc-fields", "case");
    let (mut cache, table) = probe_pivot(&case, "LV");
    let full = {
        let mut work = MAX_PIVOT_WORK;
        compute(&table, &cache, &mut work).unwrap();
        MAX_PIVOT_WORK - work
    };
    // A formula that builds a large array spends far more than the pivot itself.
    let rev = cache.fields.iter_mut().find(|f| f.name == "Rev").unwrap();
    rev.formula = Some("SUM(SEQUENCE(5000))*0+Units*Price".to_string());
    let heavy = {
        let mut work = MAX_PIVOT_WORK;
        compute(&table, &cache, &mut work).unwrap();
        MAX_PIVOT_WORK - work
    };
    assert!(heavy > full + 100_000, "{heavy} vs {full}");
    // Past the budget the pivot has no result, and the budget is spent.
    let mut work = full + (heavy - full) / 2;
    assert_eq!(
        compute(&table, &cache, &mut work),
        Err(PivotComputeError::Limit)
    );
    assert_eq!(work, 0);
}

#[test]
fn a_formula_the_engine_refuses_leaves_the_pivot_without_a_result() {
    let case = read("t4505-pivot-calc-fields", "case");
    let (mut cache, table) = probe_pivot(&case, "LV");
    let rev = cache.fields.iter_mut().find(|f| f.name == "Rev").unwrap();
    rev.formula = Some("A1*2".to_string());
    let mut work = MAX_PIVOT_WORK;
    assert_eq!(
        compute(&table, &cache, &mut work),
        Err(PivotComputeError::Malformed)
    );
}

#[test]
fn a_chain_of_calculated_fields_is_ordered_without_recursion() {
    // C0 = Units, C1 = C0+1, ..., listed last first: each is the field's sum plus its depth.
    let mut fields = vec![field("Units", None)];
    let depth = MAX_CALCULATED_FIELDS - 1;
    for i in (0..depth).rev() {
        let formula = if i == 0 {
            "Units".to_string()
        } else {
            format!("C{}+1", i - 1)
        };
        fields.push(field(&format!("C{i}"), Some(&formula)));
    }
    let data = [1];
    let mut work = MAX_PIVOT_WORK;
    let mut plan = Plan::new(&data, &fields, &mut work).unwrap().unwrap();
    assert_eq!(plan.operands, vec![0]);
    let values = plan.values(vec![Ok(5.0)], &mut work).unwrap();
    let shown = plan.shows[0].unwrap();
    assert_eq!(
        values[shown],
        Ok(5.0 + f64::from(u32::try_from(depth - 1).unwrap()))
    );
}

/// The functions a calculated field may call are every `Function` but [`REFUSED_FUNCTIONS`], a
/// denylist. Pinning the count means a new variant fails here until someone decides whether the
/// sandbox may run it (no I/O, no host state, no workbook state: CLAUDE.md directive 5) and,
/// if so, raises the count, or else adds it to the denylist.
#[test]
fn every_function_variant_is_reviewed_for_the_sandbox() {
    let call = |kind: Function| Node::FunctionKind { kind, args: vec![] };
    let (accepted, refused): (Vec<Function>, Vec<Function>) =
        Function::into_iter().partition(|kind| check_node(&call(kind.clone())).is_ok());
    let names: Vec<String> = refused.iter().map(|kind| format!("{kind:?}")).collect();
    assert_eq!(
        names,
        [
            "Lambda",
            "Let",
            "Rand",
            "Randbetween",
            "Cell",
            "Info",
            "Indirect",
            "Offset",
            "Getpivotdata",
            "Randarray",
            "Image",
            "Stockhistory",
            "Fieldvalue",
            "Filterxml",
            "Now",
            "Today",
        ]
    );
    assert_eq!(
        accepted.len(),
        492,
        "review the new functions for the sandbox"
    );
}

#[test]
fn huge_arrays_and_texts_stay_bounded() {
    // SEQUENCE and REPT past their limits are errors, and building up to them is charged to the
    // pivot's budget: the pivot has a result whose cells are errors, or none (Limit).
    let case = read("t4505-pivot-calc-fields", "case");
    for formula in [
        "SUM(SEQUENCE(1e9))+Units",
        "LEN(REPT(\"a\",1e9))+Units",
        "SUM(SEQUENCE(30000,30000))+Units",
    ] {
        let (mut cache, table) = probe_pivot(&case, "LV");
        let rev = cache.fields.iter_mut().find(|f| f.name == "Rev").unwrap();
        rev.formula = Some(formula.to_string());
        let mut work = MAX_PIVOT_WORK;
        match compute(&table, &cache, &mut work) {
            Err(error) => assert_eq!(error, PivotComputeError::Limit, "{formula}"),
            Ok(result) => {
                let rev = 2;
                let value = result.value(0, 0, rev).unwrap();
                assert!(value.is_err(), "{formula}: {value:?}");
            }
        }
    }
}

#[test]
fn quoted_names_are_sheet_prefixes_unless_switched_on() {
    let context = CellReferenceRC {
        sheet: SANDBOX_SHEET.to_string(),
        row: 1,
        column: 1,
    };
    let mut parser = new_parser_english(vec![SANDBOX_SHEET.to_string()], vec![], HashMap::new());
    assert!(parser.parse("'Unit Cost'*2", &context).has_parse_error());
    assert!(!parser.parse("'Sheet1'!A1*2", &context).has_parse_error());
    parser.set_quoted_names(true);
    assert!(matches!(
        parser.parse("'Bob''s'", &context),
        Node::NamedVariableKind { name, .. } if name == "Bob's"
    ));
    assert!(!parser.parse("'Sheet1'!A1*2", &context).has_parse_error());
}
