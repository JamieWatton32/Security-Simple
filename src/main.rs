//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use app::Database;
use rusqlite::Connection;
use simple_security::*;
fn make_directory()->std::io::Result<()>{
        std::fs::create_dir("C:\\security_simple\\")?;
        Ok(())

}
fn create_tables() -> Result<(), rusqlite::Error> {
    let conn = Connection::open("C:\\security_simple\\data")?;
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
    use key::Key;
    let _ = make_directory();
    let _ = create_tables();

    let options = eframe::NativeOptions::default();
    let key = Key::retrieve_key();
    let _ = eframe::run_native(
        "Simple Security",
        options,
        Box::new(|_| Box::new(Database::new(key))),
    );
}
