# 0336 — Round-trip delta violations ratchet in the baseline until the patch tier writes new styles

- **Status:** accepted (2026-11-07, T-4514)
- **Context:**
  - SPEC-0033 D5 says a delta violation always fails, even when the same item is in the R0 baseline.
  - T-4514's first measurement shows why that cannot hold yet. Any new style (E6) makes the save regenerate the
    whole package, so R1/E6 and R1/all violate on almost every case: 527 of 541 Calipers workbooks and all 912
    corpus-bench ones. Making violations fail outright would turn every round-trip gate red with no fix available
    in the same change.
- **Decision:**
  - R1 delta violations take the case's baseline like any other item, under their scenario (`R1/E<n>`, `R1/all`).
    A new violation is a regression, and a fixed one is ratcheted with `make eval-baseline`.
  - The rule becomes absolute ("a delta violation always fails") once T-4623 (new styles in the patch tier),
    T-4831 (a first shared-string part) and T-4832 (shared-formula groups) have burnt the known violations down. The
    change that makes it absolute amends this record.
- **Consequences:**
  - "Only the change I made" is measured and ratcheted now, not guaranteed. The R1 kept rate in the CLI line and
    the report is how far from the guarantee each suite is.
  - The fixtures baseline holds about 9,300 item lines until T-4623 lands, most of them R1/E6 and R1/all.
