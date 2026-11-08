//! The `dropped` verdict (SPEC-0033 D7.2, T-4562): a part the save did not write passes when the
//! save report names it for a reason the scenario accepts: one every scenario accepts, or, in an
//! R2 run, one its edit explains (T-5005). A reported part that is still in the output is a
//! report bug, scored `changed` (D7.3).

use std::collections::HashSet;

use rustsheet::{DropReason, SaveReport};

use super::package::Package;
use super::r2::sheet_of_part;
use super::{Item, Verdict};

/// The sheet an R2 edit acted on (T-5005), whose parts its drops may name.
#[derive(Debug, Clone, Copy)]
pub(super) struct Edited<'a> {
    /// The sheet as the source names it, which the part keys use.
    pub(super) sheet: &'a str,
    /// The sheet as the output names it (a rename's new name), which a report's `sheet` may use.
    pub(super) now: &'a str,
    /// Whether the edit deleted it.
    pub(super) deleted: bool,
}

/// Rescores the part items of one save by its report: an accepted drop of a lost part becomes
/// `dropped`, with no weight; a reported part that was kept becomes `changed`. `edited` is the
/// sheet of an R2 run's edit, `None` in any other run.
pub(super) fn apply(
    items: &mut [Item],
    source: &Package,
    report: &SaveReport,
    edited: Option<Edited<'_>>,
) {
    let reported: HashSet<(String, bool)> = (report.dropped.iter())
        .map(|d| {
            let key = source.key_of(&d.item);
            let ok = accepted(&d.reason, &key, edited);
            (key, ok)
        })
        .collect();
    for item in items.iter_mut().filter(|i| i.kind == "part") {
        let key = item.key.clone();
        let accepted_drop = reported.contains(&(key.clone(), true));
        let any = accepted_drop || reported.contains(&(key, false));
        match item.verdict {
            Verdict::Lost if accepted_drop => {
                item.verdict = Verdict::Dropped;
                item.weight = 0;
            }
            Verdict::Kept if any => {
                item.verdict = Verdict::Changed;
                item.weight = 1;
                item.detail = Some("the save report names it dropped, but it was written".into());
            }
            _ => {}
        }
    }
}

/// Whether a drop of the part keyed `key` passes (D7.2): `Policy` and
/// `ActiveContentInPlainFormat` in any scenario; in an R2 run, `OwnerDeleted` for a part of the
/// sheet the run deleted, and `StaleAfterStructuralEdit` naming the edited sheet for a part of it.
/// `StaleAfterEdit` passes only where R1's `Δ(E)` explains it, which T-5004 adds with its
/// triggers, so it stays `lost` until then.
fn accepted(reason: &DropReason, key: &str, edited: Option<Edited<'_>>) -> bool {
    let of_edited = |e: &Edited<'_>| sheet_of_part(key) == Some(e.sheet);
    match reason {
        DropReason::Policy | DropReason::ActiveContentInPlainFormat => true,
        DropReason::OwnerDeleted => edited.is_some_and(|e| e.deleted && of_edited(&e)),
        DropReason::StaleAfterStructuralEdit { sheet } => {
            edited.is_some_and(|e| (sheet == e.sheet || sheet == e.now) && of_edited(&e))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write as _};

    use rustsheet::{DropReason, Dropped, SaveReport, SaveTier};

    use super::super::package::Package;
    use super::super::{Item, Verdict};
    use super::apply;

    fn package() -> Package {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let files = [
            (
                "_rels/.rels",
                r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml" Target="customXml/item1.xml"/></Relationships>"#,
            ),
            ("xl/workbook.xml", "<workbook/>"),
            (
                "xl/_rels/workbook.xml.rels",
                r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/calcChain" Target="calcChain.xml"/></Relationships>"#,
            ),
            ("xl/calcChain.xml", "<calcChain/>"),
            ("customXml/item1.xml", "<root/>"),
        ];
        for (name, text) in files {
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .expect("start");
            zip.write_all(text.as_bytes()).expect("write");
        }
        let bytes = zip.finish().expect("finish").into_inner();
        Package::read(&bytes).expect("package")
    }

    fn part(key: &str, verdict: Verdict) -> Item {
        Item {
            scenario: "R0",
            kind: "part",
            key: key.into(),
            verdict,
            detail: None,
            candidate: None,
            weight: usize::from(verdict == Verdict::Lost),
        }
    }

    #[test]
    fn an_accepted_drop_passes_and_a_reported_kept_part_is_changed() {
        let pkg = package();
        let chain = pkg.key_of("XL/calcChain.xml");
        let custom = pkg.key_of("customXml/item1.xml");
        let mut items = vec![part(&chain, Verdict::Lost), part(&custom, Verdict::Lost)];
        let report = SaveReport::new(
            SaveTier::Verbatim,
            vec![
                Dropped::new("xl/calcChain.xml", DropReason::Policy),
                Dropped::new("customXml/item1.xml", DropReason::NotPreserved),
            ],
        );
        apply(&mut items, &pkg, &report, None);
        // calcChain by policy passes; a part not preserved stays lost.
        assert_eq!(
            (items[0].verdict, items[0].weight, items[1].verdict),
            (Verdict::Dropped, 0, Verdict::Lost)
        );
        // A part the report names but the save wrote is a report bug (D7.3).
        let mut kept = vec![part(&custom, Verdict::Kept)];
        apply(&mut kept, &pkg, &report, None);
        assert_eq!(kept[0].verdict, Verdict::Changed);
    }
}
