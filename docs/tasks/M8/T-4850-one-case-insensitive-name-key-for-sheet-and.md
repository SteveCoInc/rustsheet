---
id: T-4850
title: "One case-insensitive name key for sheet and defined names across the engine and the facade"
milestone: M8
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Sheet names and defined names have one case-insensitive key each, used by every lookup, duplicate check and the facade.

## Context
Upper-case vs lower-case folding disagree ("Maß" vs "MASS", Kelvin sign U+212A). Sheet names: the engine upper-cases (`sheet_names_match` types.rs, `get_sheet_index_by_name` model.rs, new_empty.rs duplicate check, sheet_undo.rs); the facade lower-cases (`resolve_sheet_by_name`, crates/rustsheet/src/refs.rs). Defined names: lookups use `NameKey::new` → `to_lowercase` (name_key.rs); existence and duplicate checks compare `to_uppercase` (defined_names.rs, model.rs).

Found by the T-4856 duplicate-concept audit (one definition per concept, CLAUDE.md directive 3). Excel decides each rule (directive 1): where the existing variants disagree and no probe in `eval/` or `docs/excel-semantics/` settles which one Excel follows, probe it first (`excel-probe` skill) and record the finding.

## Acceptance criteria
- [ ] One key type/function per name kind; the `to_uppercase`/`to_lowercase` comparisons above are gone.
- [ ] Probe: does Excel treat sheets/names "Maß"/"MASS" and "K"/U+212A as the same? Tests pin the probed answer.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).
