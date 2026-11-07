//! Builds a column where every cell reads the one above it:
//!
//!   A1 = 1
//!   A2 = A1+1
//!   A3 = A2+1
//!   ...
//!
//! This is `deep_chain` the right way up. Natural order meets A1 first, a
//! constant, and every formula after it finds the cell it reads already
//! evaluated, so the recursion is never deeper than one formula, however long
//! the column is. It is evaluated and saved as xlsx.
//!
//! Usage:
//!   cargo run --release --example `forward_chain` -- [rows] [file]
//!
//! Defaults: 1,000,000 rows, `forward-chain.xlsx`.

mod chain;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    chain::run("forward-chain.xlsx", |model, rows| {
        model.set_user_input(0, 1, 1, "1".to_string())?;
        for row in 2..=rows {
            model.set_user_input(0, row, 1, format!("=A{}+1", row - 1))?;
        }
        Ok(())
    })
}
