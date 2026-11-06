---
id: T-4181
title: "Common Crawl round-trip corpus: index query, range fetch, census and coverage-guided selection (roundtrip-cc)"
milestone: M15
lane: harness
status: blocked
priority: P3
size: M
tier: opus
depends_on: [T-4172]
---

## Goal
A fetched `roundtrip-cc` suite of a few hundred modern, Excel-saved real-world workbooks from Common Crawl, chosen to
cover the most fidelity features. A larger pool is kept for offline scoring and fuzz seeds.

## Context
Plan: docs/research/xlsx-round-trip-2026-11-04.md §6.2b (owner suggestion, 2026-11-04). The steps:
1. Query the columnar URL index for xlsx, xlsm and xltx MIME types, skipping `content_truncated` records.
2. Range-fetch single WARC records and dedupe by hash.
3. Run the census over 20k–50k candidates.
4. Select greedily by feature coverage, weighting Excel-authored and recent files and against near-duplicate
   templates.

Fetch, never vendor (copyright unknown): pin each file by WARC filename, offset and sha256 (decision 0019 pattern).
Commit no cell contents (personal data). Oracle use waits on owner decision D7.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275). Also needs network access: `index.commoncrawl.org` and
`data.commoncrawl.org` are refused by the agent environment's network policy (2026-11-04). Proof it is available:
`curl -sSI https://index.commoncrawl.org/collinfo.json` returns 200.

## Acceptance criteria
- [ ] The suite fetches reproducibly, runs in the round-trip harness and has a baseline.
- [ ] The selection script and its coverage report are committed.
- [ ] Unseen `ext` URIs and part types are listed in the M15 research notes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Uploading to the oracle before D7 is decided.
