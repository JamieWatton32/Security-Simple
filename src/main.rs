#![warn(clippy::all, rust_2018_idioms)]
#![warn(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use app::Database;
use rusqlite::Connection;
use Simple_Security::*;

fn create_tables() -> Result<(), rusqlite::Error> {
    let conn = Connection::open("C:/Users/jamie/passwords/data")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS passwords (
            id INTEGER PRIMARY KEY,
            site TEXT NOT NULL,
            password TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS master_password (
            id INTEGER PRIMARY KEY,
            password TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let _ = create_tables();
        
    //todo. Bad practice. dotn store your key in the fucking source code
    let key: Vec<u8> = Vec::from([
        92, 44, 212, 25, 234, 193, 46, 94, 3, 243, 96, 140, 98, 42, 55, 228, 190, 114, 127, 117,
        180, 126, 153, 37, 44, 140, 191, 115, 192, 249, 90, 71,
    ]);

    let options = eframe::NativeOptions::default();

    let _ = eframe::run_native(
        "Simple Security",
        options,
        Box::new(|_| Box::new(Database::new(key))),
    );
}
