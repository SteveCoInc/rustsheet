//! A round-trip baseline keyed by case and scenario (T-4961, decision 0345).
//!
//! The ratchet takes a failing item of a case whose checked count changed as new coverage, not as
//! a regression. A round-trip case's count used to be summed over its scenarios, so adding a
//! scenario changed every case's count and let a re-record take an existing scenario's
//! regressions for new coverage. #3892 (T-4514) added R1 that way, and it recorded
//! `roundtrip-poi/54436.xlsx`'s R0 loss from #3800 as a known failure.
//!
//! So every line names its case and scenario, `<case>#<scenario>`, with that scenario's own
//! checked count. A new scenario is a new key, new coverage. A regression in a scenario whose own
//! count held stays a regression.
//!
//! A committed baseline written before this change has no `#` in its keys. It is judged against
//! the old per-case form ([`super::baseline`]), so the first re-record that writes the new form
//! accepts nothing the old rule refused.

use std::collections::BTreeMap;

use super::{case_line, CaseResult, CaseStatus};
use crate::baseline::Baseline;
use crate::suite::BaselineMode;

/// Separates a case from its scenario in a baseline key.
const SEPARATOR: char = '#';

/// The baseline key of `case`'s `scenario`.
fn key(case: &str, scenario: &str) -> String {
    format!("{case}{SEPARATOR}{scenario}")
}

/// Whether `old` was written per case rather than per scenario: a key without `#`.
#[must_use]
pub fn is_legacy(old: &Baseline) -> bool {
    old.checked.keys().any(|case| !case.contains(SEPARATOR))
}

/// The results as a baseline with one key per case and scenario (module docs). Each key
/// counts the items its scenario compared. A case that did not run is one `!Status` failure, on
/// the first scenario it would have run (`R0`). Otherwise [`super::baseline`]'s rules apply per
/// scenario.
#[must_use]
pub fn baseline(results: &[&CaseResult], mode: BaselineMode) -> Baseline {
    let mut b = Baseline::default();
    if mode == BaselineMode::None {
        return b;
    }
    for r in results {
        let mut failing: BTreeMap<&str, Vec<&super::Item>> = BTreeMap::new();
        if r.status == CaseStatus::Fail {
            for i in r.items.iter().filter(|i| !i.verdict.passes()) {
                failing.entry(i.scenario).or_default().push(i);
            }
        }
        let mut counts: BTreeMap<&str, usize> = r.scenarios.iter().map(|(s, n)| (*s, *n)).collect();
        if counts.is_empty() {
            counts.insert(super::scenarios::R0, r.checked);
        }
        // A failing item names a scenario that compared something; a count of 0 keeps its key
        // anyway, so the item is never left without one.
        for s in failing.keys() {
            counts.entry(s).or_insert(0);
        }
        for (s, n) in &counts {
            b.checked.insert(key(&r.id, s), *n);
        }
        match r.status {
            CaseStatus::Pass => {}
            CaseStatus::Fail => {
                for (s, items) in failing {
                    let case = key(&r.id, s);
                    if mode == BaselineMode::Cases {
                        b.failing.insert((case, case_line(items.into_iter())));
                    } else {
                        for i in items {
                            b.failing.insert((case.clone(), i.baseline_key()));
                        }
                    }
                }
            }
            other => {
                let first = counts
                    .keys()
                    .next()
                    .copied()
                    .unwrap_or(super::scenarios::R0);
                b.failing.insert((key(&r.id, first), format!("!{other:?}")));
            }
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::super::{diff, Item, Verdict};
    use super::*;

    fn item(scenario: &'static str, key: &str) -> Item {
        Item {
            scenario,
            kind: "element",
            key: key.to_string(),
            verdict: Verdict::Lost,
            detail: None,
            candidate: None,
            weight: 3,
        }
    }

    /// A failing case that compared `counts` items per scenario and lost `items`.
    fn case(counts: &[(&'static str, usize)], items: Vec<Item>) -> CaseResult {
        CaseResult {
            id: "poi/a.xlsx".to_string(),
            suite: "roundtrip-poi".to_string(),
            status: if items.is_empty() {
                CaseStatus::Pass
            } else {
                CaseStatus::Fail
            },
            message: None,
            millis: 0,
            checked: counts.iter().map(|(_, n)| n).sum(),
            scenarios: counts.iter().copied().collect(),
            skipped: BTreeMap::new(),
            items,
        }
    }

    /// The case of #3892: a scenario is added while R0 newly loses an item. Per case, the summed
    /// count changed and the loss passed as new coverage; per scenario it is a regression.
    #[test]
    fn a_new_scenario_does_not_hide_an_existing_scenarios_regression() {
        for mode in [BaselineMode::Cases, BaselineMode::Cells] {
            let before = case(&[("R0", 10)], Vec::new());
            let after = case(&[("R0", 10), ("R1", 12)], vec![item("R0", "/a/b")]);
            let per_case = diff(
                &super::super::baseline(&[&before], mode),
                &super::super::baseline(&[&after], mode),
            );
            assert!(
                per_case.regressions.is_empty(),
                "the old rule's hole: {per_case:?}"
            );
            let delta = diff(&baseline(&[&before], mode), &baseline(&[&after], mode));
            assert_eq!(delta.regressions.len(), 1, "{mode:?}: {delta:?}");
            assert!(
                delta.regressions[0].starts_with("poi/a.xlsx#R0\t"),
                "{delta:?}"
            );
            assert_eq!(delta.new_cases, vec!["poi/a.xlsx#R1".to_string()]);
        }
    }

    /// A new scenario's own failures are new coverage, as they should be.
    #[test]
    fn a_new_scenarios_failures_are_new_coverage() {
        for mode in [BaselineMode::Cases, BaselineMode::Cells] {
            let before = case(&[("R0", 10)], vec![item("R0", "/a/b")]);
            let after = case(
                &[("R0", 10), ("R1", 12)],
                vec![item("R0", "/a/b"), item("R1", "/c")],
            );
            let delta = diff(&baseline(&[&before], mode), &baseline(&[&after], mode));
            assert!(delta.regressions.is_empty(), "{mode:?}: {delta:?}");
            assert_eq!(delta.new_failures.len(), 1, "{mode:?}: {delta:?}");
        }
    }

    #[test]
    fn keys_carry_the_scenario_and_its_own_count() {
        let r = case(&[("R0", 10), ("R1", 12)], vec![item("R1", "/c")]);
        let b = baseline(&[&r], BaselineMode::Cases);
        assert_eq!(
            b.checked,
            BTreeMap::from([
                ("poi/a.xlsx#R0".to_string(), 10),
                ("poi/a.xlsx#R1".to_string(), 12)
            ])
        );
        assert_eq!(b.failing.len(), 1);
        assert!(b
            .failing
            .iter()
            .all(|(c, line)| c == "poi/a.xlsx#R1" && line.starts_with("* 3 ")));
        assert!(!is_legacy(&b));
        assert!(is_legacy(&super::super::baseline(
            &[&r],
            BaselineMode::Cases
        )));
    }

    #[test]
    fn a_case_that_did_not_run_fails_on_its_first_scenario() {
        let mut r = case(&[], Vec::new());
        r.status = CaseStatus::Timeout;
        let b = baseline(&[&r], BaselineMode::Cases);
        assert_eq!(b.checked.keys().collect::<Vec<_>>(), ["poi/a.xlsx#R0"]);
        assert!(b
            .failing
            .contains(&("poi/a.xlsx#R0".to_string(), "!Timeout".to_string())));
    }
}
