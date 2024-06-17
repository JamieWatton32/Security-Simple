use crate::encryption::encryption;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::Error as SqErr;

use crate::{encryption::encryption::*, key::Key};

//Contains data for master section
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct MasterArea {
    pub key: Vec<u8>,
    pub password: String,
    pub passed: bool,
}

impl MasterArea {
    pub fn new() -> Self {
        let key = Key::retrieve_key().hex;
        MasterArea {
            key,
            password: String::new(),
            passed: false,
        }
    }

    pub fn add_master(&self, password: &str) -> Result<(), SqErr> {
        let connection = Connection::open("C:\\security_simple\\data")?;

        let encrypted_password = encryption::encrypt(password, &self.key);
        let encrypted_password_hex = hex::encode(encrypted_password);
        let mut db =
            connection.prepare("INSERT INTO master_password (id, password) VALUES (?1, ?2);")?;
        db.execute(rusqlite::params![1, encrypted_password_hex])?;
        Ok(())
    }

    fn check_master<'a>(&'a self, master_password: &'a str) -> Result<&str, SqErr> {
        let connection = Connection::open("C:\\security_simple\\data")?;
        let mut db = connection.prepare("SELECT password FROM master_password Where id = (?1);")?;
        let mut rows = db.query(rusqlite::params![1])?;

        if let Ok(Some(row)) = rows.next() {
            let encrypted_password_hex: String = row.get(0)?;

            let decrypted_password = decrypt_string(encrypted_password_hex, &self.key);

            match decrypted_password {
                Ok(d) => {
                    if d.as_str() == master_password {
                        return Ok(master_password);
                    } else {
                        Ok("Passwords did not match.")
                    }
                }
                Err(_) => return Ok("Passwords did not match."),
            }
        } else {
            Ok("Query returned no result.")
        }
    }

    pub fn extract_master(&self) -> String {
        if let Ok(p) = self.check_master(&self.password) {
            p.to_owned()
        } else {
            String::from("Invalid Password!")
        }
    }

    pub fn master_exists(&self) -> bool {
        let conn = Connection::open("C:\\security_simple\\data").unwrap();
        let mut stmt = conn
            .prepare("SELECT EXISTS(SELECT 1 FROM master_password WHERE id = ?)")
            .unwrap();
        let exists: bool = stmt.query_row(params![1], |row| row.get(0)).unwrap();

        exists
    }
}
