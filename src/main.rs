#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use app::Database;
use rusqlite::Connection;
use simple_security::*;
use encryption::key::StoreKey;
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

    let _ = make_directory();
    let _ = create_tables();

    let encryption_key = match StoreKey::retrieve_key(){
        Some(encryption_key) => {
            StoreKey::decrypt_data(&encryption_key)
        }
        None => {
            let new_key = StoreKey::make_key();
            let encrypted_key = StoreKey::encrypt_data(&new_key);
            return StoreKey::store_key(&encrypted_key);
            
        } 
    };
    
    

    let options = eframe::NativeOptions::default();

    let _ = eframe::run_native(
        "Simple Security",
        options,
        Box::new(|_| Box::new(Database::new(encryption_key))),
    );
}
