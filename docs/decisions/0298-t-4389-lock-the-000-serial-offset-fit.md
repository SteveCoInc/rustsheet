# 0298 — T-4389: lock the .000 serial offset fit in the FITTED ratchet

- **Status:** accepted (2026-11-05, T-4389)
- **Context:**
  - T-4389 found that under `.000` Excel cuts the half-stepped serial as if 2^-72 days (1.83E-17 s) were added
    first (formatter/elapsed.rs `millisecond_offset`; docs/excel-semantics/number-formats.md "Fraction digits of a
    second at a tie"; docs/research/t4389). The constant is a fit: every offset in [1.74E-17, 1.86E-17] s scores the
    538 `.000` cells of the t4323, t4332 and t4370 fraction cases, and 2^-72 days is the power of two in that window.
    Whether it sits in the serial (days) or the remainder (seconds) is a structural choice.
  - The R4 guards of decision 0188 (bit-exact text; the 4-ulp and 15-digit columns do not apply to TEXT):
    - The hold-out is a new probe, probes/t4389-holdout (368 cells), not a 30% split, because every existing `.000`
      cell was in the fit. It was generated before scoring, with a 10-point margin fixed beforehand. The model scores
      368/368, T-4370's model 337 and the correctly rounded value 81. Its 31 cells after whole seconds chose days over
      seconds.
    - After the freeze: probes/t4389-fraction-fresh scores 120/120 (T-4370 42, correctly rounded 39), and
      probes/t4389-confirm, drawn after review, scores 69/80 (51, 31).
      - Its 40 untouched separating cells confirm days over seconds (40/40 against 0/40).
      - In the band [2^-19, 2^-18) days the offset is half an ulp and rounds to even. Excel follows that in 29 of 40
        cells; the other 11 are T-4419's.
    - The flips the constant rests on sit 36 to 40 ulp of the serial below the tie, above the 4-ulp
      reproducibility floor. The window's edges are set at the ulp scale on excel-graph alone.
    - The tag is `FITTED(T-4389, 538 cells, offset 1.74E-17..1.86E-17 s, serial below 2^-18 days, .000 only)`. Above
      2^-18 days the double addition leaves the serial unchanged, so the code falls back to T-4370's model.
- **Decision:** add the new tag to `quality/fitted.json` (`scripts/check_fitted.py --update`), so formatter/elapsed.rs
  goes from 0 to 1 tag.
- **Consequences:**
  - The ratchet now counts this fit. Removing it, or changing its range, needs the same guards again.
  - T-4419 owns the 11 band cells of probes/t4389-confirm. A desktop recheck of t4370-ulp-map-2 A75 and A76 joins
    A417, A418 and t4333-count-ulp-scan E6 in M14 Phase 4 (decision 0039).
