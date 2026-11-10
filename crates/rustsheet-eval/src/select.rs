//! Which suites a run scores, and which external suites `fetch` downloads (SPEC-0035).
//!
//! A suite fetched into eval/.cache (poi, calipers, corpus-bench), and a round-trip suite over
//! one's cache (`cases_of`), is skipped while its cache is missing or stale. With
//! `--require-fetched` (T-4725, SPEC-0035 D2 "belt and braces") a selected *gate* suite in that
//! state fails the run instead: CI fetches the gate suites itself, so a skip there is a broken
//! cache path or a renamed suite, never something to pass over.

use anyhow::{bail, Result};

use crate::fetch::fetch_missing;
use crate::suite::{FetchState, Suite};

/// The suites and limits of a run (`--suite`, `--filter`, `--timeout`, the fetch flags).
#[derive(clap::Args, Clone, Debug, Default)]
pub struct Selection {
    /// Restrict to these suites (repeatable). Default: all suites (or gate suites for check).
    #[arg(long = "suite")]
    pub suites: Vec<String>,
    /// Only cases whose id contains this substring.
    #[arg(long)]
    pub filter: Option<String>,
    /// Per-case timeout in seconds.
    #[arg(long, default_value_t = 60)]
    pub timeout: u64,
    /// Fetch each selected external suite whose cache is missing or stale before scoring it,
    /// instead of skipping it. A failed fetch skips that suite and the run continues.
    #[arg(long)]
    pub fetch_missing: bool,
    /// Fail, naming each, when a selected gate suite (`gate = true`) is not fetched or its cache
    /// is stale, instead of skipping it (SPEC-0035 D2). An ungated suite is still skipped.
    #[arg(long, conflicts_with = "fetch_missing")]
    pub require_fetched: bool,
}

/// What [`select_from`] chose: the suites to score, and the lines for the caller to print.
#[derive(Debug, Default)]
pub struct Selected {
    pub suites: Vec<Suite>,
    /// `fetched <suite> (<n> files)`, for stdout (`--fetch-missing`).
    pub fetched: Vec<String>,
    /// `skipping <suite>: ...`, for stderr: a cache missing, stale or failing to fetch.
    pub skipped: Vec<String>,
}

/// The suites of `all` that `sel` selects (`gate_only`: only the gate suites when `--suite` names
/// none) and that can be scored: a suite whose cache is missing or stale is fetched
/// (`--fetch-missing`) or skipped.
///
/// # Errors
/// When nothing is selected, or (`--require-fetched`) when a selected gate suite is not fetched.
pub fn select_from(all: Vec<Suite>, sel: &Selection, gate_only: bool) -> Result<Selected> {
    let mut unfetched_gates = Vec::new();
    let (mut fetched, mut skipped) = (Vec::new(), Vec::new());
    let chosen: Vec<Suite> = all
        .into_iter()
        .filter(|s| {
            let selected = if sel.suites.is_empty() {
                !gate_only || s.manifest.gate
            } else {
                sel.suites.contains(&s.manifest.name)
            };
            if s.fetch_state() == FetchState::NotFetched {
                if sel.fetch_missing && selected {
                    return match fetch_missing(s) {
                        Ok(files) => {
                            fetched.push(format!("fetched {} ({files} files)", s.manifest.name));
                            true
                        }
                        Err(skip) => {
                            skipped.push(skip);
                            false
                        }
                    };
                }
                // A round-trip suite over a cell suite's cache is fetched as that suite.
                let fetch_as = s.manifest.cases_of.as_ref().unwrap_or(&s.manifest.name);
                if sel.require_fetched && selected && s.manifest.gate {
                    unfetched_gates
                        .push(format!("{} (`fetch --suite {fetch_as}`)", s.manifest.name));
                    return false;
                }
                skipped.push(format!(
                    "skipping {}: not fetched (run `rustsheet-eval fetch --suite {fetch_as}`)",
                    s.manifest.name
                ));
                return false;
            }
            selected
        })
        .collect();
    if !unfetched_gates.is_empty() {
        bail!(
            "--require-fetched: gate suites not fetched or stale: {}",
            unfetched_gates.join(", ")
        );
    }
    if chosen.is_empty() {
        bail!("no suites selected");
    }
    Ok(Selected {
        suites: chosen,
        fetched,
        skipped,
    })
}

/// `rustsheet-eval fetch`'s choice of suites: the external suites of `all` (a `[fetch]` spec)
/// named by `names` (every one when it is empty), only the gate suites with `gates`, and only
/// those whose cache is missing or stale with `missing` (SPEC-0035 D3's pre-fetch,
/// `fetch --missing --gates`). The second list holds the external suites `missing` leaves alone
/// because their cache is current.
#[must_use]
pub fn to_fetch<'a>(
    all: &'a [Suite],
    names: &[String],
    missing: bool,
    gates: bool,
) -> (Vec<&'a Suite>, Vec<&'a Suite>) {
    all.iter()
        .filter(|s| {
            s.manifest.fetch.is_some()
                && (names.is_empty() || names.contains(&s.manifest.name))
                && (!gates || s.manifest.gate)
        })
        .partition(|s| !missing || s.fetch_state() == FetchState::NotFetched)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::suite::{SuiteManifest, FETCH_MARKER};

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rs-select-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    /// An external suite `name` under `dir`, gated or not, its cache fetched at its pin or not.
    fn external(dir: &Path, name: &str, gate: bool, fetched: bool) -> Suite {
        let text = format!(
            "name = \"{name}\"\nkind = \"spec\"\nlicense = \"MIT\"\ngate = {gate}\n[fetch]\n\
             url = \"file:///nowhere.zip\"\nsha256 = \"{}\"\ninclude = [\"**/*.case.json\"]\n",
            "0".repeat(64)
        );
        let suite = Suite {
            manifest: toml::from_str::<SuiteManifest>(&text).expect("manifest"),
            dir: dir.join(name),
            cache_dir: dir.join("cache").join(name),
        };
        if fetched {
            let pin = suite.manifest.fetch.as_ref().expect("fetch").pin();
            fs::create_dir_all(&suite.cache_dir).expect("mkdir");
            fs::write(suite.cache_dir.join(FETCH_MARKER), pin).expect("marker");
        }
        suite
    }

    fn names(suites: &[impl std::borrow::Borrow<Suite>]) -> Vec<&str> {
        suites
            .iter()
            .map(|s| s.borrow().manifest.name.as_str())
            .collect()
    }

    fn required(suites: &[&str]) -> Selection {
        Selection {
            suites: suites.iter().map(ToString::to_string).collect(),
            require_fetched: true,
            ..Selection::default()
        }
    }

    #[test]
    fn an_unfetched_gate_suite_fails_require_fetched_and_names_every_one() {
        let dir = scratch("unfetched");
        let all = || {
            vec![
                external(&dir, "poi", true, false),
                external(&dir, "bench", true, false),
                external(&dir, "calipers", false, false),
                external(&dir, "fixtures", true, true),
            ]
        };
        let err = select_from(all(), &required(&[]), true).expect_err("unfetched gates");
        let msg = format!("{err:#}");
        assert!(msg.contains("poi (`fetch --suite poi`)"), "{msg}");
        assert!(msg.contains("bench (`fetch --suite bench`)"), "{msg}");
        assert!(!msg.contains("calipers"), "{msg}");
        // Without the flag the same selection skips them, as before.
        let plain = Selection::default();
        assert_eq!(
            names(&select_from(all(), &plain, true).expect("skip").suites),
            ["fixtures"]
        );
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn fetched_gate_suites_and_unfetched_ungated_ones_pass_require_fetched() {
        let dir = scratch("fetched");
        let all = vec![
            external(&dir, "poi", true, true),
            external(&dir, "calipers", false, false),
        ];
        // `check` selects the gate suites; calipers is ungated and skipped as before.
        let chosen = select_from(all, &required(&[]), true).expect("fetched");
        assert_eq!(names(&chosen.suites), ["poi"]);
        assert_eq!(chosen.skipped.len(), 1, "calipers is skipped with its line");
        // Named explicitly, an unfetched ungated suite is still skipped, not an error.
        let all = vec![
            external(&dir, "poi", true, true),
            external(&dir, "calipers", false, false),
        ];
        let chosen = select_from(all, &required(&["poi", "calipers"]), false).expect("named");
        assert_eq!(names(&chosen.suites), ["poi"]);
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn a_stale_gate_cache_fails_require_fetched() {
        let dir = scratch("stale");
        let poi = external(&dir, "poi", true, true);
        fs::write(poi.cache_dir.join(FETCH_MARKER), "an older pin").expect("marker");
        let err = select_from(vec![poi], &required(&["poi"]), false).expect_err("stale");
        assert!(format!("{err:#}").contains("poi"), "{err:#}");
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn an_unselected_unfetched_gate_suite_does_not_fail_require_fetched() {
        let dir = scratch("unselected");
        let all = vec![
            external(&dir, "poi", true, false),
            external(&dir, "fixtures", true, true),
        ];
        let chosen = select_from(all, &required(&["fixtures"]), false).expect("not selected");
        assert_eq!(names(&chosen.suites), ["fixtures"]);
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn require_fetched_conflicts_with_fetch_missing() {
        #[derive(clap::Parser)]
        struct Cli {
            #[command(flatten)]
            sel: Selection,
        }
        use clap::Parser;
        let err = Cli::try_parse_from(["t", "--require-fetched", "--fetch-missing"])
            .err()
            .expect("conflict");
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
        assert!(Cli::try_parse_from(["t", "--require-fetched"]).is_ok());
    }

    #[test]
    fn fetch_missing_gates_picks_the_missing_or_stale_gate_suites() {
        let dir = scratch("plan");
        let stale = external(&dir, "bench", true, true);
        fs::write(stale.cache_dir.join(FETCH_MARKER), "an older pin").expect("marker");
        let all = vec![
            external(&dir, "poi", true, false),
            stale,
            external(&dir, "current", true, true),
            external(&dir, "calipers", false, false),
        ];
        let none: &[String] = &[];
        let (fetch, current) = to_fetch(&all, none, true, true);
        assert_eq!(
            (names(&fetch), names(&current)),
            (vec!["poi", "bench"], vec!["current"])
        );
        // `--gates` alone: every gate suite, current or not; never calipers.
        let (fetch, current) = to_fetch(&all, none, false, true);
        assert_eq!(names(&fetch), ["poi", "bench", "current"]);
        assert!(current.is_empty());
        // `--missing` alone: ungated suites too.
        let (fetch, _) = to_fetch(&all, none, true, false);
        assert_eq!(names(&fetch), ["poi", "bench", "calipers"]);
        // Neither: every external suite, as `fetch` always did; `--suite` still narrows it.
        let (fetch, _) = to_fetch(&all, none, false, false);
        assert_eq!(fetch.len(), 4);
        let (fetch, _) = to_fetch(&all, &["calipers".to_string()], false, false);
        assert_eq!(names(&fetch), ["calipers"]);
        fs::remove_dir_all(&dir).expect("cleanup");
    }
}
