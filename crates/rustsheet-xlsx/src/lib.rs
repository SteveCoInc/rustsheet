//! RustSheet xlsx import and export for the calculation engine.
//!
//! This crate is internal. Its API is inherited from the IronCalc fork; applications open and
//! save workbooks through the `rustsheet` crate (`rustsheet::Workbook::open`, `save_xlsx`),
//! which applies the load limits and typed errors this crate does not.
//!
//! A simple example with some numbers, a new sheet and a formula:
//!
//!
//! ```rust,no_run
#![doc = include_str!("../examples/hello_calc.rs")]
//! ```
//!
//! ## Examples
//!
//! This is a collection of full fledged examples you can use as a starting point or for learning purposes.
//! You might find the code in the examples folder
//!
//! ### Styling the workbook
//!
//! Adding colors, to cells, full columns or full rows is easy
//!
//! ```rust,no_run
#![doc = include_str!("../examples/hello_styles.rs")]
//! ```
//!
//! ### Changing column width and row height
//!
//! ```rust,no_run
#![doc = include_str!("../examples/widths_and_heights.rs")]
//! ```

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/ironcalc/ironcalc/main/assets/logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/ironcalc/ironcalc/main/assets/favicon.ico"
)]

pub mod compare;
pub mod error;
pub mod export;
pub mod import;
pub mod util;
pub use rustsheet_engine as base;

pub mod metadata_clock;
