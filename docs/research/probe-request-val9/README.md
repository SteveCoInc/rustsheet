# Probe request val9 (2026-11-11): one combined fill, after the M4 request

All of these are excel-graph fills. They come from several lanes, and are batched so there is one run instead of six.

1. **LINEST R4.2 (CR-321684429, decision 0188 R4.2)**: five cases, 861 checked cells. See LINEST-R42.md for the commands.
   - eval/suites/probes/cases/t2574-zero-head-fresh.case.json: 55 formulas plus the Precision canary; 42 of them separate the old and new rule. Frozen predictions for both rules: docs/research/t2574/zero-head-fresh-predictions.tsv.
   - eval/accuracy/tables/{LINEST,TREND,LOGEST,GROWTH}/r4-T-2574.case.json, fresh from the standard generator, unfilled.
2. **Probe cases that have no oracle sidecar yet** (eval/suites/probes/cases/): address-empty-args, aggregate-array-form-3d-span-hidden, date-text-spaced-hyphens, defined-name-body-scope-first-sheet, eng-impower-modulus-choice, legacy-array-parameters-linest, legacy-index-match-one-row, legacy-unary-plus-aggregate-argument, range-to-a-call-cells, sheet-span-cell-like-names-typed, structured-ref-digit-column, t0481-curly-quote-names-c, typed-if-one-argument, val3-fvs-05, val3-fvs-06, val3-fvs-08; plus at-unary-sign-range and t-n-at-unary-plus-if-array (P559459515).
   `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes` fills every case that has no sidecar.
3. **Stress typed values (T-5447)**: docs/research/probe-request-val9/stress-t5447/ holds one case, 60 cells, plus its README. It reads typed values through =Cn formulas. Run it by path:
   `cd tools && uv run rustsheet-oracle recalc -b excel-graph ../docs/research/probe-request-val9/stress-t5447/t5447-typed-value-readers.case.json`

Wanted back: the *.oracle.json sidecars for 1 and 2, and the printed JSON for 3.
4. **F.INV / BETA.INV search fresh sweep (T-5107, P559210060, decision 0188 R4.2)**: one case, 526 cells, eval/suites/probes/cases/t5107-search-fresh.case.json; frozen values of both engines in docs/research/t5107-fin-val4/search-fresh-predictions.tsv; details in T5107-SEARCH-FRESH.md. The probes-suite fill picks it up.
5. **F.INV / BETA.INV search, the tails (T-5107, R4.2 full domain)**: one case, 287 cells: inverse probabilities above 0.981 and below 1E-300, small, large and extreme shapes. See T5107-SEARCH-FRESH-TAILS.md. The probes-suite fill picks it up.
