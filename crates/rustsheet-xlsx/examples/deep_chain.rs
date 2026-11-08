//! Builds a column where every cell reads the one below it:
//!
//!   A1 = A2+1
//!   A2 = A3+1
//!   ...
//!   A(n) = 1
//!
//! Natural order meets A1 first, and A1 needs the whole column before it can
//! be computed, so a plain recursive evaluation goes n formulas deep. This is
//! the workbook that used to overflow the stack. this would easily produce a
//! stack overflow with a naive recursive evaluation.
//! It is evaluated and saved as xlsx.
//!
//! Usage:
//!   cargo run --release --example `deep_chain` -- [rows] [file]
//!
//! Defaults: 1,000,000 rows, `deep-chain.xlsx`.

mod chain;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    chain::run("deep-chain.xlsx", |model, rows| {
        for row in 1..rows {
            model.set_user_input(0, row, 1, format!("=A{}+1", row + 1))?;
        }
        model.set_user_input(0, rows, 1, "1".to_string())
    })
}
