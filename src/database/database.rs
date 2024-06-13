use rusqlite::{Connection, Error as SqErr};
use crate::encryption::*;
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Database<'a> {
    fname: &'a str,
    key: Vec<u8>,

}

#[derive(Debug)]
pub enum PassErr {
    DbErr(SqErr),
    EncryptError,
}

impl From<SqErr> for PassErr {
    fn from(s: SqErr) -> Self {
        PassErr::DbErr(s)
    }
}

impl<'a> Database<'a> {
    pub fn new(fname: &'a str, key: Vec<u8>) -> Self {
        Self { fname, key,}
    }
    pub fn add(&self, site_name: &str, password: &str) -> Result<(), PassErr> {
        let connection = Connection::open(&self.fname)?;
        self.create_table(&connection)?;
        let encrypted_password = encryption::encrypt(password, &self.key);
        let encrypted_password_hex = hex::encode(encrypted_password);
        let mut db =
            connection.prepare("INSERT INTO passwords (site, password) VALUES (?1, ?2);")?;
        db.execute(rusqlite::params![site_name, encrypted_password_hex])?;

        Ok(())
    }

    fn create_table(&self, conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS passwords (
                id INTEGER PRIMARY KEY,
                site TEXT NOT NULL,
                password TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn drop(&self, site_name: &str) -> Result<(), PassErr> {
        let connection = Connection::open(&self.fname)?;
        let mut db = connection.prepare("DELETE FROM passwords WHERE site = ?1;")?;
        db.execute(rusqlite::params![site_name])?;

        Ok(())
    }

    pub fn get(&self, site_name: &str) -> Result<String, PassErr> {
        let connection = Connection::open(&self.fname)?;

        let mut db = connection.prepare("SELECT password FROM passwords WHERE site = ?1;")?;
        let mut rows = db.query(rusqlite::params![site_name])?;

        if let Some(row) = rows.next()? {
            let encrypted_password_hex: String = row.get(0)?;
            let encrypted_password =
                hex::decode(encrypted_password_hex).map_err(|_| PassErr::EncryptError)?;
            let decrypted_password = encryption::decrypt(&encrypted_password, &self.key);
            return Ok(decrypted_password);
        } else {
            println!("test");
            return Err(PassErr::DbErr(rusqlite::Error::QueryReturnedNoRows));
        }
    }
}
