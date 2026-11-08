//! R2's sheet edits (T-5005): the key translation of a renamed sheet, the expected delta's
//! rules, and whole runs of each edit over a two-sheet workbook.

use rustsheet::{DropReason, Dropped, SaveReport, SaveTier, Workbook, WorkbookOptions};

use super::super::dropped;
use super::super::package::Package;
use super::super::scenarios::run_scenarios;
use super::super::{Item, Verdict};
use super::{
    explained, fact_sheet, sheet_of_part, swap_facts, swap_part, Delta, Rename, SheetEdit,
    Structural, R2_SHEET,
};
use crate::roundtrip::semantic::Facts;

#[test]
fn a_fact_names_its_sheet_whatever_the_name_holds() {
    assert_eq!(fact_sheet("sheet A B order"), Some("A B"));
    assert_eq!(fact_sheet("cell Q!1 x!B2 formula"), Some("Q!1 x"));
    assert_eq!(fact_sheet("cf S!A1 C3:D4 priority 2"), Some("S"));
    assert_eq!(fact_sheet("name S!Print_Area"), Some("S"));
    assert_eq!(fact_sheet("name Total"), None);
    assert_eq!(fact_sheet("table T1"), None);
}

#[test]
fn a_rename_swaps_only_the_named_sheet() {
    let facts: Facts = [
        ("sheet A order", "0"),
        ("sheet A B order", "1"),
        ("cell A!B2 input", "1"),
        ("cell A B!B2 input", "2"),
        ("table T", "A!A1:B2 header 1"),
        ("table U", "A B!A1:B2 header 1"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    let swapped = swap_facts(facts, "A", "N");
    let keys: Vec<&str> = swapped.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        [
            "cell A B!B2 input",
            "cell N!B2 input",
            "sheet A B order",
            "sheet N order",
            "table T",
            "table U"
        ]
    );
    assert_eq!(swapped["table T"], "N!A1:B2 header 1");
    assert_eq!(swapped["table U"], "A B!A1:B2 header 1");
}

#[test]
fn a_part_key_names_its_sheet_by_the_workbook_relationship_only() {
    let key = "/officeDocument[0]/worksheet[A]/drawing[0]";
    assert_eq!(sheet_of_part(key), Some("A"));
    assert_eq!(sheet_of_part("/officeDocument[0]/styles[0]"), None);
    // A deeper sheet-typed relationship is keyed by ordinal, not by a sheet's name.
    assert_eq!(sheet_of_part("/x[0]/y[0]/worksheet[A]"), None);
    assert_eq!(
        swap_part(key, "A", "N"),
        "/officeDocument[0]/worksheet[N]/drawing[0]"
    );
    assert_eq!(swap_part(key, "B", "N"), key);
}

fn item(kind: &'static str, key: &str, verdict: Verdict, detail: Option<&str>) -> Item {
    Item {
        scenario: "R2/x",
        kind,
        key: key.into(),
        verdict,
        detail: detail.map(str::to_string),
        candidate: None,
        weight: 1,
    }
}

fn structural(edit: SheetEdit) -> Structural {
    let sheet = if edit == SheetEdit::Add {
        R2_SHEET
    } else {
        "A"
    };
    Structural {
        edit,
        sheet: sheet.into(),
        renamed: (edit == SheetEdit::Rename).then(|| Rename {
            from: "A".into(),
            to: R2_SHEET.into(),
        }),
        cells: Delta::default(),
    }
}

/// One expected-delta case: the edit, the item's kind, its key (`W` stands for the workbook part's
/// key, `R2` for the new sheet name), verdict and detail, and whether the edit explains it.
type Case = (
    SheetEdit,
    &'static str,
    &'static str,
    Verdict,
    Option<&'static str>,
    bool,
);

fn check(cases: &[Case]) {
    for &(edit, kind, key, verdict, detail, expected) in cases {
        let key = key
            .replace('W', "/officeDocument[0]")
            .replace("R2", R2_SHEET);
        let detail = detail.map(|d| d.replace("R2", R2_SHEET));
        let it = item(kind, &key, verdict, detail.as_deref());
        assert_eq!(
            explained(&it, &structural(edit)),
            expected,
            "{edit:?} {key}"
        );
    }
}

#[test]
fn a_sheet_edit_explains_the_package_items_it_rewrites() {
    use SheetEdit::{Add, Delete, Move, Rename};
    use Verdict::{Changed, Lost};
    let names = "W:/workbook/definedNames[0]/definedName[N,]";
    let scoped = "W:/workbook/definedNames[0]/definedName[N,0]";
    let tab = "/officeDocument[0]/worksheet[B]:/worksheet/sheetViews[0]/sheetView[0]@tabSelected";
    let titles = "/extended-properties[0]:/Properties/ep:TitlesOfParts[0]";
    let title = "/extended-properties[0]:/Properties/ep:TitlesOfParts[0]/vt:vector[0]/vt:lpstr[0]";
    check(&[
        // The sheet list and book views follow every edit.
        (
            Add,
            "attribute",
            "W:/workbook/sheets[0]/sheet[1]@name",
            Changed,
            None,
            true,
        ),
        (
            Rename,
            "attribute",
            "W:/workbook/bookViews[0]/workbookView[0]@activeTab",
            Changed,
            None,
            true,
        ),
        // A defined name that now names the renamed sheet; one that changed otherwise is not.
        (
            Rename,
            "element",
            names,
            Changed,
            Some("text \"'R2'!A1\" became x"),
            true,
        ),
        (
            Rename,
            "element",
            names,
            Changed,
            Some("text \"1\" became 2"),
            false,
        ),
        // A move renumbers sheet-scoped names only; a delete also rewrites to #REF!.
        (Move, "element", scoped, Lost, None, true),
        (Move, "element", names, Lost, None, false),
        (
            Delete,
            "element",
            names,
            Changed,
            Some("text \"A!B1\" became #REF!"),
            true,
        ),
        // Adding a sheet rewrites no name.
        (
            Add,
            "element",
            "W:/workbook/definedNames[0]",
            Verdict::Added,
            None,
            false,
        ),
        // The selection moves with a delete or move, not a rename.
        (Delete, "attribute", tab, Changed, None, true),
        (Rename, "attribute", tab, Changed, None, false),
        // The sheet titles change inside; a list lost whole is the export's loss.
        (Rename, "element", title, Changed, None, true),
        (Rename, "element", titles, Lost, None, false),
    ]);
}

#[test]
fn a_sheet_edit_explains_the_facts_and_parts_of_its_sheet() {
    use SheetEdit::{Add, Delete, Move, Rename};
    use Verdict::{Added, Changed, Lost};
    check(&[
        // An added sheet's facts and parts are new; another sheet's are not.
        (Add, "semantic", "sheet R2 order", Added, None, true),
        (Add, "part", "W/worksheet[R2]", Added, None, true),
        (Add, "part", "W/worksheet[B]", Added, None, false),
        // A deleted sheet's facts go with it; its parts pass only as dropped.
        (Delete, "semantic", "cell A!B2 input", Lost, None, true),
        (Delete, "semantic", "cell B!B2 input", Lost, None, false),
        (Delete, "part", "W/worksheet[A]", Lost, None, false),
        // Tab order shifts under a delete or move only.
        (Move, "semantic", "sheet B order", Changed, None, true),
        (Rename, "semantic", "sheet B order", Changed, None, false),
    ]);
}

/// A two-sheet workbook: `A` holds a constant, `B` a formula and a sheet-scoped name naming it.
fn two_sheets() -> Vec<u8> {
    let mut wb = Workbook::new(&WorkbookOptions::default()).expect("workbook");
    wb.edit(|t| {
        let first = t.first_sheet()?;
        t.rename_sheet(first, "A")?;
        let b = t.add_sheet("B")?;
        t.set("A!A1", 2.0)?;
        t.set("B!A1", "=A!A1*3")?;
        t.define_sheet_name(b, "Base", "=A!$A$1")
    })
    .expect("seeded");
    wb.recalculate();
    wb.to_xlsx_bytes(rustsheet::SaveOptions::default())
        .expect("saved")
}

#[test]
fn a_renamed_sheet_keeps_its_items_and_only_its_name_changes() {
    let bytes = two_sheets();
    let run = run_scenarios(
        &bytes,
        None,
        &["R2/rename".into(), "R2/move".into(), "R2/add".into()],
        false,
    )
    .ok()
    .expect("runs");
    // The rewritten formula and name, the sheet list and the renamed sheet's own items are all
    // the edits' doing: no semantic item fails.
    let semantic: Vec<&Item> = (run.items.iter())
        .filter(|i| i.scenario.starts_with("R2") && i.kind == "semantic")
        .collect();
    assert!(semantic.is_empty(), "{semantic:#?}");
    for label in ["R2/rename", "R2/move", "R2/add"] {
        assert!(run.checked.contains_key(label), "{label} ran");
    }
}

#[test]
fn a_deleted_sheet_loses_only_its_parts_until_the_report_names_them() {
    let bytes = two_sheets();
    let run = run_scenarios(&bytes, None, &["R2/delete".into()], false)
        .ok()
        .expect("runs");
    let delete: Vec<&Item> = (run.items.iter())
        .filter(|i| i.scenario == "R2/delete")
        .collect();
    // B's formula became #REF! and its name went with A's position: explained. A's worksheet
    // part is lost until T-5004 reports it OwnerDeleted.
    assert!(delete.iter().all(|i| i.kind != "semantic"), "{delete:#?}");
    let parts: Vec<&str> = (delete.iter())
        .filter(|i| i.kind == "part")
        .map(|i| i.key.as_str())
        .collect();
    assert_eq!(parts, ["/officeDocument[0]/worksheet[A]"]);
    // A one-sheet workbook cannot lose its only sheet: delete and move are n/a.
    let mut one = Workbook::new(&WorkbookOptions::default()).expect("workbook");
    one.edit(|t| t.set("Sheet1!A1", 1.0)).expect("seeded");
    let one = one
        .to_xlsx_bytes(rustsheet::SaveOptions::default())
        .expect("saved");
    let run = run_scenarios(&one, None, &["R2/delete".into(), "R2/move".into()], false)
        .ok()
        .expect("runs");
    assert!(!run.checked.contains_key("R2/delete") && !run.checked.contains_key("R2/move"));
}

#[test]
fn an_r2_drop_passes_only_for_the_edited_sheet() {
    let bytes = two_sheets();
    let pkg = Package::read(&bytes).expect("package");
    let a = pkg.key_of("xl/worksheets/sheet1.xml");
    let b = pkg.key_of("xl/worksheets/sheet2.xml");
    let lost = |key: &str| Item {
        weight: 1,
        ..item("part", key, Verdict::Lost, None)
    };
    let report = |reason: DropReason| {
        SaveReport::new(
            SaveTier::Regenerated {
                reason: "test".into(),
            },
            vec![
                Dropped::new("xl/worksheets/sheet1.xml", reason.clone()),
                Dropped::new("xl/worksheets/sheet2.xml", reason),
            ],
        )
    };
    let deleted = dropped::Edited {
        sheet: "A",
        now: "A",
        deleted: true,
    };
    let verdicts = |reason: DropReason, edited| {
        let mut items = vec![lost(&a), lost(&b)];
        dropped::apply(&mut items, &pkg, &report(reason), edited);
        (items[0].verdict, items[1].verdict)
    };
    // OwnerDeleted passes for the deleted sheet's part only, and only in a delete.
    assert_eq!(
        verdicts(DropReason::OwnerDeleted, Some(deleted)),
        (Verdict::Dropped, Verdict::Lost)
    );
    assert_eq!(
        verdicts(DropReason::OwnerDeleted, None),
        (Verdict::Lost, Verdict::Lost)
    );
    let moved = dropped::Edited {
        deleted: false,
        ..deleted
    };
    assert_eq!(
        verdicts(DropReason::OwnerDeleted, Some(moved)),
        (Verdict::Lost, Verdict::Lost)
    );
    // StaleAfterStructuralEdit passes when it names the edited sheet, by either name.
    let renamed = dropped::Edited {
        sheet: "A",
        now: R2_SHEET,
        deleted: false,
    };
    let stale = |sheet: &str| DropReason::StaleAfterStructuralEdit {
        sheet: sheet.into(),
    };
    assert_eq!(
        verdicts(stale(R2_SHEET), Some(renamed)),
        (Verdict::Dropped, Verdict::Lost)
    );
    assert_eq!(
        verdicts(stale("B"), Some(renamed)),
        (Verdict::Lost, Verdict::Lost)
    );
}
