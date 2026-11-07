// rt <in.xlsx> <out.xlsx> [edit]   -> load, optionally edit an empty far cell, evaluate, save
use rustsheet_xlsx::{export::save_to_xlsx, import::load_from_xlsx};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let r = std::panic::catch_unwind(|| {
        let mut m = match load_from_xlsx(&a[1], "en", "UTC", "en") { Ok(m) => m, Err(e) => { println!("LOADERR\t{e}"); std::process::exit(2) } };
        if a.len() > 3 { let _ = m.set_user_input(0, 1000, 50, "42".to_string()); }
        if std::env::var_os("NOEVAL").is_none() { m.evaluate(); }
        let _ = std::fs::remove_file(&a[2]);
        if let Err(e) = save_to_xlsx(&m, &a[2]) { println!("SAVEERR\t{e}"); std::process::exit(3) }
    });
    if r.is_err() { println!("PANIC"); std::process::exit(4) }
}
