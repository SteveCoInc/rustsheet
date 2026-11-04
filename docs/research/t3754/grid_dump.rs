// T-3754 scratch: paste into `fisher.rs`'s `mod tests` (not compiled from here), run
// `T3754_GRID=points.txt T3754_OUT=engine.txt cargo test -p rustsheet_engine --lib --release t3754_dump -- --ignored`,
// then remove it. Writes "d1 d2 x upper lower" per grid point, from `f_tail`.
#[test]
#[ignore = "T-3754 research scratch"]
fn t3754_dump() {
    use std::fmt::Write as _;
    let grid = std::fs::read_to_string(std::env::var("T3754_GRID").unwrap()).unwrap();
    let mut out = String::new();
    for line in grid.lines() {
        let v: Vec<f64> = line.split_whitespace().map(|s| s.parse().unwrap()).collect();
        let (d1, d2, x) = (v[0], v[1], v[2]);
        let upper = f_tail(d1, d2, x, true);
        let lower = f_tail(d1, d2, x, false);
        writeln!(out, "{d1} {d2} {x} {upper:e} {lower:e}").unwrap();
    }
    std::fs::write(std::env::var("T3754_OUT").unwrap(), out).unwrap();
}
