//! What `deep_chain` and `forward_chain` share: argument parsing, timing, the report and the
//! save. Each example only fills column A.

use std::io::Write;
use std::time::Instant;

use rustsheet_xlsx::{base::Model, export::save_to_xlsx};

/// Reads `[rows] [file]` (defaults 1,000,000 and `default_file`), builds column A with `fill`,
/// evaluates, prints the timings and the first and last values, and saves the workbook as xlsx.
pub fn run(
    default_file: &str,
    fill: impl Fn(&mut Model, i32) -> Result<(), String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let rows: i32 = match args.next() {
        Some(value) => value.parse()?,
        None => 1_000_000,
    };
    let file_name = args.next().unwrap_or_else(|| default_file.to_string());
    if !(1..=1_048_576).contains(&rows) {
        return Err("rows must be between 1 and 1,048,576".into());
    }
    let mut out = std::io::stdout().lock();
    let mut model = Model::new_empty(&file_name, "en", "UTC", "en")?;

    let start = Instant::now();
    fill(&mut model, rows)?;
    writeln!(out, "built {rows} rows in {:?}", start.elapsed())?;

    let start = Instant::now();
    model.evaluate();
    writeln!(out, "evaluated in {:?}", start.elapsed())?;
    let last = model.get_formatted_cell_value(0, rows, 1)?;
    writeln!(
        out,
        "A1 = {}, A{rows} = {last}",
        model.get_formatted_cell_value(0, 1, 1)?
    )?;

    // save_to_xlsx refuses to overwrite a file
    if std::path::Path::new(&file_name).exists() {
        std::fs::remove_file(&file_name)?;
    }
    let start = Instant::now();
    save_to_xlsx(&model, &file_name)?;
    let kib = std::fs::metadata(&file_name)?.len() / 1024;
    writeln!(
        out,
        "saved {file_name} ({kib} KiB) in {:?}",
        start.elapsed()
    )?;
    Ok(())
}
