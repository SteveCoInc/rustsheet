//! `rustsheet-eval package-diff` (T-4890): another writer's save of a workbook scored against
//! ours with the round-trip differ, as R0 scores an output against its source. The Excel re-save
//! probe compares each of our saves with Excel's re-save of it.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::Context as _;

use super::package::Package;
use super::{scenarios, score, Item, Sides, Verdict};

/// The arguments of `rustsheet-eval package-diff`.
#[derive(clap::Args, Debug, Clone)]
pub struct PackageDiff {
    /// Our package (the source side).
    pub ours: PathBuf,
    /// The other writer's save of it.
    pub theirs: PathBuf,
    /// Print the items as JSON instead of one line each.
    #[arg(long)]
    pub json: bool,
}

impl PackageDiff {
    /// The failing items, one per line or as one JSON array, and a one-line summary.
    ///
    /// # Errors
    /// When a file cannot be read, or `ours` is not a readable package or does not load.
    pub fn report(&self) -> anyhow::Result<(String, String)> {
        let read = |p: &Path| std::fs::read(p).with_context(|| p.display().to_string());
        let name = self
            .ours
            .file_name()
            .map(|n| n.to_string_lossy().into_owned());
        let (items, checked) = package_diff(
            &read(&self.ours)?,
            &read(&self.theirs)?,
            name.as_deref().unwrap_or("workbook"),
        )?;
        let failing: Vec<&Item> = items
            .iter()
            .filter(|i| i.verdict != Verdict::Kept)
            .collect();
        let mut out = String::new();
        if self.json {
            out = serde_json::to_string(&failing)? + "\n";
        } else {
            for i in &failing {
                let detail = i.detail.as_deref().unwrap_or("");
                let _ = writeln!(out, "{} {} {} {detail}", i.verdict.label(), i.kind, i.key);
            }
        }
        let lost: usize = failing.iter().map(|i| i.weight).sum();
        let n = failing.len();
        Ok((
            out,
            format!("{checked} items compared, {lost} not kept, {n} failing"),
        ))
    }
}

/// Scores `theirs`, another writer's save of `ours`, as R0 scores an output against its source:
/// validator problems, parts, elements and attributes, then the semantic facts, with our cached
/// values as the saved side (not recalculated: two packages are compared, not our engine against
/// one). Returns the items and how many of ours were compared (T-4890).
///
/// # Errors
/// When `ours` is not a readable package or does not load.
pub fn package_diff(ours: &[u8], theirs: &[u8], name: &str) -> anyhow::Result<(Vec<Item>, usize)> {
    let source = Package::read(ours)?;
    let saved = scenarios::load_stored(ours, Some(name)).map_err(|e| match e {
        scenarios::R0Error::Source(e) | scenarios::R0Error::Engine(e) => anyhow::anyhow!(e),
    })?;
    let sides = Sides {
        source: &source,
        source_bytes: ours,
        saved: &saved,
        name,
    };
    Ok(score(scenarios::R0, &sides, theirs))
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read as _, Write as _};

    use super::super::Verdict;
    use super::package_diff;

    const NAME: &str = "merged-covered-values.xlsx";

    fn fixture() -> Vec<u8> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../eval/suites/fixtures/cases");
        std::fs::read(root.join(NAME)).expect("fixture")
    }

    /// `bytes` with the zip entry `part` left out.
    fn without(bytes: &[u8], part: &str) -> Vec<u8> {
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).expect("zip");
        let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).expect("entry");
            let name = entry.name().expect("name").to_string();
            if name == part {
                continue;
            }
            let mut data = Vec::new();
            entry.read_to_end(&mut data).expect("read");
            out.start_file(name, zip::write::SimpleFileOptions::default())
                .expect("start");
            out.write_all(&data).expect("write");
        }
        out.finish().expect("finish").into_inner()
    }

    /// Two packages are compared, not our engine against one: a workbook against itself is kept
    /// whole, even where our engine computes other values than its cache holds (merged covered
    /// cells).
    #[test]
    fn a_workbook_against_itself_finds_nothing() {
        let bytes = fixture();
        let (items, checked) = package_diff(&bytes, &bytes, NAME).expect("diff");
        let failing: Vec<_> = items
            .iter()
            .filter(|i| i.verdict != Verdict::Kept)
            .collect();
        assert!(checked > 0 && failing.is_empty(), "{failing:?}");
    }

    #[test]
    fn a_part_the_other_save_dropped_is_lost() {
        let bytes = fixture();
        let (items, _) =
            package_diff(&bytes, &without(&bytes, "docProps/app.xml"), NAME).expect("diff");
        assert!(
            items.iter().any(|i| i.kind == "part"
                && i.verdict == Verdict::Lost
                && i.key.contains("extended-properties")),
            "{items:?}"
        );
    }
}
