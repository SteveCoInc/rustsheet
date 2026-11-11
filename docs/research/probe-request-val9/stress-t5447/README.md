# Probe request: the stress suite's typed values, through readers (T-5447, poc-stress-cc)

One case, `t5447-typed-value-readers.case.json`, for `eval/suites/probes/cases/`. It has 60 cells on sheet S:
- C1:C30 are the 30 typed values the stress suite still has no oracle value for, each typed as one entry after
  load;
- D1:D30 are their readers, `=C1` .. `=C30`, each typed right after its source (row-major order).

Wanted: excel-graph values (`rustsheet-oracle fill -b excel-graph`), as for t5313.

Why a second case: the t5313 fill (main 50f418fed) recorded formula cells only. So
`t5313-stress-session-inputs` holds no value for its 29 typed values in column C, and the addendum's typed `#N/A`
has none either. A reader hands on the typed cell's value and type as they are, and the stress suite's evidence
index points at the reader once its value lands (T-5447).

Map of cell -> stress case id and reader: `cells.tsv`. The case validates (`rustsheet-eval show`: status Pending,
no oracle yet). The inputs are the same typed values as t5313's C1:C29 and the addendum's E1, in the stress suite's
order, so nothing here is new input.
