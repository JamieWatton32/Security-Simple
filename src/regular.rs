use rusqlite::Error as SqErr;
use rusqlite::MappedRows;
use rusqlite::Row;
use rusqlite::{params, Connection, Result};

use crate::{encryption::encryption::*, key::Key};

#[derive(Debug)]
pub enum DbErr {
    DbErr(SqErr),
    EncryptError,
}

impl From<SqErr> for DbErr {
    fn from(s: SqErr) -> Self {
        DbErr::DbErr(s)
    }
}

//contains data for site/pw section
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RegularArea {
    key: Vec<u8>,
    pub passwords: Vec<String>,
    pub sites: Vec<String>,
    pub ids: Vec<i32>,
}

//Publics
impl RegularArea {
    pub fn new() -> Self {
        let key = Key::retrieve_key().hex;
        RegularArea {
            key,
            passwords: Vec::new(),
            sites: Vec::new(),
            ids: Vec::new(),
        }
    }
    pub fn add_regular(&self, site_name: &str, password: &str) -> Result<(), SqErr> {
        let conn = Connection::open("C:\\security_simple\\data").unwrap();
        let encryption_key = &self.key;
        let (epassword, esite) = (
            encrypt(&password, &encryption_key),
            encrypt(&site_name, &encryption_key),
        );

        let (site_hex, password_hex) = (hex::encode(esite), hex::encode(epassword));

        let mut db = conn.prepare("INSERT INTO passwords (site, password) VALUES (?1, ?2);")?;
        db.execute(params![site_hex, password_hex])?;
        Ok(())
    }

    pub fn fetch_from_db(&mut self) -> Result<(), String> {
        if let Ok(sites) = self.fetch_sites() {
            for site in sites {
                self.sites.push(site);
            }
        }
        if let Ok(passwords) = self.fetch_passwords() {
            for pw in passwords {
                self.passwords.push(pw);
            }
        }
        if let Ok(ids) = self.fetch_ids() {
            for i in ids {
                self.ids.push(i);
            }
        }
        Ok(())
    }

    pub fn generate_password(&self) -> String {
        use passwords::PasswordGenerator;
        let pg = PasswordGenerator::new()
            .length(6)
            .numbers(true)
            .lowercase_letters(true)
            .uppercase_letters(true)
            .symbols(true)
            .spaces(false)
            .exclude_similar_characters(true)
            .strict(true);

        pg.generate_one().unwrap()
    }

    pub fn remove_entry(&self, id: usize) -> Result<usize> {
        let conn = Connection::open("C:\\security_simple\\data").unwrap();
        let mut stmt = conn
            .prepare("DELETE FROM passwords WHERE ID = ?1;")
            .unwrap();

        match stmt.execute(params![id]) {
            Ok(rows_affected) => Ok(rows_affected),
            Err(e) => Err(e),
        }
    }
}

//Internals
impl RegularArea {
    fn fetch_passwords(&mut self) -> Result<Vec<String>, SqErr> {
        let conn = Connection::open("C:\\security_simple\\data").unwrap();
        let mut stmt = conn.prepare("SELECT password FROM passwords").unwrap();
        let encrypted_iter = stmt.query_map([], |row| {
            let password: String = row.get(0)?;
            Ok(password)
        });
        let passwords = self.build_decrypted_array(encrypted_iter);
        Ok(passwords)
    }

    fn fetch_sites(&mut self) -> Result<Vec<String>, SqErr> {
        let conn = Connection::open("C:\\security_simple\\data").unwrap();
        let mut stmt = conn.prepare("SELECT site FROM passwords")?;
        let site_iter = stmt.query_map([], |row| {
            let site: String = row.get(0)?;
            Ok(site)
        });

        let sites = self.build_decrypted_array(site_iter);

        Ok(sites)
    }

    fn fetch_ids(&mut self) -> Result<Vec<i32>> {
        let conn = Connection::open("C:\\security_simple\\data").unwrap();
        let mut stmt = conn.prepare("SELECT id FROM passwords")?;
        let mut rows = stmt.query(params![])?;

        let mut ids = Vec::new();
        while let Some(row) = rows.next()? {
            let id: i32 = row.get(0)?;
            ids.push(id);
        }
        Ok(ids)
    }

    fn build_decrypted_array(
        &mut self,
        encrypted_iter: Result<MappedRows<impl FnMut(&Row) -> Result<String, SqErr>>, SqErr>,
    ) -> Vec<String> {
        let mut decrypted_iter = Vec::new();
        match encrypted_iter {
            Ok(e) => {
                for each in e {
                    match each {
                        Ok(hex) => {
                            if let Ok(decrypted) = decrypt_string(hex, &self.key) {
                                decrypted_iter.push(decrypted);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        decrypted_iter
    }
}
