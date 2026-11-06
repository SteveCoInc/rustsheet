# T-4482 predictions (engine, before the oracle fill)

The engine's values for probes/t4482-chained-call-arity and probes/t4482-chained-call-arity-optional, read from
`rustsheet-eval show` on origin/main (commit before the fill), committed before the excel-graph fill as the task asks.

Columns: A `ERROR.TYPE(X)`, B `IFERROR(X,"e")`, C `ISERROR(X)`, D `TEXTJOIN(",",FALSE,MAP({1;2},LAMBDA(v,ERROR.TYPE(X))))`.

## t4482-chained-call-arity

| row | X | A | B | C | D |
|---|---|---|---|---|---|
| 1 | `LAMBDA(x,LAMBDA(y,x+y))(1)()` | 3 | e | TRUE | 3,3 |
| 2 | `LAMBDA(x,LAMBDA(y,x+y))(1)(2,3)` | 3 | e | TRUE | 3,3 |
| 3 | `LAMBDA(x,LAMBDA(y,x+y))(1,2)(3)` | 3 | e | TRUE | 3,3 |
| 4 | `LAMBDA(x,LAMBDA(y,x+y))()(3)` | 3 | e | TRUE | 3,3 |
| 5 | `LAMBDA(x,LAMBDA(y,x+y))(1/0)()` | 3 | e | TRUE | 3,3 |
| 6 | `LAMBDA(x,LAMBDA(y,x+y))(1)(NA(),2)` | 3 | e | TRUE | 3,3 |
| 7 | `LET(f,LAMBDA(x,LAMBDA(y,x*y)),f(3)())` | 3 | e | TRUE | 3,3 |
| 8 | `MYFN(3)(4,5)` | 3 | e | TRUE | 3,3 |
| 9 | `MYFN(3)()` | 3 | e | TRUE | 3,3 |
| 10 | `LAMBDA(x,LAMBDA(y,x+y))(1)(2)` (control) | #N/A | 3 | FALSE | #N/A |

## t4482-chained-call-arity-optional

| row | X | A | B | C | D |
|---|---|---|---|---|---|
| 1 | `LAMBDA(x,LAMBDA(y,[z],x+y+IF(ISOMITTED(z),0,z)))(1)(2)` | #N/A | 3 | FALSE | #N/A |
| 2 | `..(1)()` | 3 | e | TRUE | 3,3 |
| 3 | `..(1)(2,3)` | #N/A | 6 | FALSE | #N/A |

So the engine says every wrong count, at any link and beside an error argument, is `#VALUE!` (an argument error is
not passed on first: rows 5 and 6).
