# 0318 — Mark CHIINV s1 A93 under the huge-shape closed-form R3 case, on decision 0289's terms

- **Status:** accepted (2026-11-06, T-4104)
- **Context:** T-4104's CHIINV `s1` table (excel-graph, 2026-11-06) has one point outside the envelope:
  `CHIINV(1.44764470787181E-250,2585098457)` (A93). Excel gives 2588216316.627514, the 50-digit quantile is
  2587528759.268203: the Wilson-Hilferty closed form, 2.66e-4 relative off (against G = 1e-4). That is decision 0188 R3's
  case 3, statistics.md "GAMMA.INV at a huge shape, closed-form branch", which names CHIINV. The point's shape (df/2) is
  1.29E9, below the 3.9E9 lowest shape decision 0289 recorded for the branch: new evidence of where Excel takes it, not a
  wider case (the case is the branch, wherever Excel takes it). Marking a point is a protected-path change (SPEC-0029).
- **Decision:** A93 goes into `eval/accuracy/math-over-excel.tsv` under that entry, as decision 0289 did for the
  e-T-3863 batch. No R3 case is added or widened, so no new owner ruling is needed. The engine still returns Excel's
  closed form there (through `WILSON_HILFERTY_SPREAD`, outside its FITTED range), so the point stays a known failure owned
  by T-3364, which moves the branch to the math.
- **Consequences:** Gate A checks A93 against the math only. Its known-failure line goes when T-3364 lands. The entry's
  evidence records the shape-1.29E9 point.
