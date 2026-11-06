# 0314 — T-3858: lock the CHISQ.INV.RT df 1 #NUM! cutoff in the FITTED ratchet

- **Status:** accepted (2026-11-06, T-3858)
- **Context:**
  - T-3858 found that `CHISQ.INV.RT(p,1)` and `CHIINV(p,1)` return `#NUM!` for p <= 8.3253E-17 and 68.7632522116684
    from 8.3254E-17 up, while `CHISQ.INV(1-p,1)` returns a value throughout. So the cutoff is the right-tail
    functions' own check on p (docs/excel-semantics/statistics.md, "CHISQ.INV.RT and CHIINV quirks";
    probes-numeric/t3858-chisq-df1-num-cutoff). No closed form for the edge was found, so the constant
    `ONE_DEGREE_RIGHT_TAIL_P_MIN = 8.32535E-17` in functions/statistical/chisq.rs, the bracket's midpoint, is a fit.
  - The R4 guards of decision 0188:
    - Hold-out, fixed before scoring: 49/49 cells, against 17/49 for the old rule.
    - After the freeze: probes-numeric/t3858-chisq-df1-num-cutoff-fresh scores 80/80.
    - The bracket [8.3253E-17, 8.3254E-17] is set by excel-graph values. Both ends are literals Excel keeps exactly
      at 15 digits, so the oracle's reproducibility floor does not apply.
    - The tag is `FITTED(T-3858, 180 cells, df 1, p 5.5E-17..1.12E-16)`.
- **Decision:** add the tag to `quality/fitted.json` (`scripts/check_fitted.py --update`, 92 to 93 tags).
- **Consequences:**
  - The ratchet now counts this fit. Removing it, or changing its range, needs the same guards again.
  - T-3337 keeps the df 1 landing above the edge. A df 1.5 probe at 8.3253E-17 and 8.3254E-17 would confirm the
    unit test's inferred row A9.
