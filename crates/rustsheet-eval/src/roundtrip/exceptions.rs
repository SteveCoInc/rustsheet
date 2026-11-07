//! Per-case exceptions of a round-trip suite (SPEC-0033 D8): narrow tokens that excuse one
//! baseline item of one case, each owned by a task or a decision. The self-test is the run itself:
//! an exception that excuses nothing is stale, and a gate run fails on it.

use anyhow::{bail, Context};
use serde::Deserialize;

use super::CaseResult;
use crate::runner::CaseStatus;
use crate::suite::Suite;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    exception: Vec<Exception>,
}

/// One exception: the case id, the excused item as its baseline line names it, and the task
/// (`T-1234`) or decision (`decision 0123`) that owns it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Exception {
    pub(super) case: String,
    pub(super) item: String,
    pub(super) owner: String,
}

/// The exceptions of `suite`, or none when it names no file.
///
/// # Errors
/// When the file cannot be read or parsed, or an exception names no owner.
pub(super) fn load(suite: &Suite) -> anyhow::Result<Vec<Exception>> {
    let Some(file) = &suite.manifest.exceptions else {
        return Ok(Vec::new());
    };
    // A file next to the manifest, never one outside the suite.
    let inside = std::path::Path::new(file)
        .components()
        .all(|c| matches!(c, std::path::Component::Normal(_)));
    if !inside {
        bail!(
            "{}: exceptions must name a file inside the suite directory",
            suite.dir.display()
        );
    }
    let path = suite.dir.join(file);
    let text = std::fs::read_to_string(&path).with_context(|| path.display().to_string())?;
    let parsed: File = toml::from_str(&text).with_context(|| path.display().to_string())?;
    for e in &parsed.exception {
        if !owned(&e.owner) {
            bail!(
                "{}: the exception for {} names no task or decision as its owner",
                path.display(),
                e.case
            );
        }
    }
    Ok(parsed.exception)
}

/// `T-<digits>` or `decision <4 digits>`.
fn owned(owner: &str) -> bool {
    let digits = |s: &str, n: Option<usize>| {
        !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && n.is_none_or(|n| s.len() == n)
    };
    owner.strip_prefix("T-").is_some_and(|d| digits(d, None))
        || owner
            .strip_prefix("decision ")
            .is_some_and(|d| digits(d, Some(4)))
}

/// Removes from `results` the items `exceptions` excuse (a case whose every item then passes is
/// `Pass`). Returns the exceptions that excused nothing, which are stale, and how many items were
/// excused, which the run reports so that an exception is never silent.
pub(super) fn apply<'a>(
    results: &mut [&mut CaseResult],
    exceptions: &'a [Exception],
) -> (Vec<&'a Exception>, usize) {
    let mut used = vec![false; exceptions.len()];
    let mut excused = 0;
    for r in results.iter_mut() {
        let before = r.items.len();
        r.items.retain(|item| {
            let key = item.baseline_key();
            let hit = exceptions
                .iter()
                .position(|e| e.case == r.id && e.item == key);
            if let Some(i) = hit {
                used[i] = true;
                excused += 1;
            }
            hit.is_none()
        });
        if r.items.len() != before
            && r.status == CaseStatus::Fail
            && r.items.iter().all(|i| i.verdict.passes())
        {
            r.status = CaseStatus::Pass;
        }
    }
    let stale = exceptions
        .iter()
        .zip(used)
        .filter(|(_, used)| !used)
        .map(|(e, _)| e)
        .collect();
    (stale, excused)
}
