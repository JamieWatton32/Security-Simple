use crate::encryption::encryption;
use rusqlite::Connection;
use rusqlite::Error as SqErr;


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

//Contains data for master section
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct MasterArea {
    pub key: Vec<u8>,
    pub password: String,
    pub passed: bool,
    pub created: bool,
}

//contains data for site/pw section
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RegularArea {
    key: Vec<u8>,
    pub passwords: Vec<String>,
    pub sites: Vec<String>,
}

impl RegularArea {
    pub fn new() -> Self {
        let key = Key::retrieve_key().hex;
        RegularArea {
            key,
            passwords: Vec::new(),
            sites: Vec::new(),
        }
    }
    pub fn add_regular(&self, site_name: &str, password: &str) -> Result<(), rusqlite::Error> {
        let conn = Connection::open("C:\\security_simple\\data").unwrap();
        let encryption_key = &self.key;
        let encrypted_password = encrypt(&password, &encryption_key);
        let encrypted_password_hex = hex::encode(encrypted_password);
        let mut db = conn.prepare("INSERT INTO passwords (site, password) VALUES (?1, ?2);")?;
        db.execute(rusqlite::params![site_name, encrypted_password_hex])?;
        Ok(())
    }

    fn fetch_sites(&self) -> Result<Vec<String>, rusqlite::Error> {
        let conn = Connection::open("C:\\security_simple\\data").unwrap();
        let mut stmt = conn.prepare("SELECT site FROM passwords")?;
        let site_iter = stmt.query_map([], |row| {
            let site: String = row.get(0)?;
            Ok(site)
        });

        let mut sites_iter = Vec::new();
        match site_iter {
            Ok(e) => {
                for each in e {
                    match each {
                        Ok(s) => {
                            sites_iter.push(s);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        Ok(sites_iter)
    }
    fn fetch_passwords(&self) -> Result<Vec<String>, rusqlite::Error> {
        let conn = Connection::open("C:\\security_simple\\data").unwrap();
        let mut stmt = conn.prepare("SELECT password FROM passwords").unwrap();
        let encrypted_iter = stmt.query_map([], |row| {
            let password: String = row.get(0)?;
            Ok(password)
        });

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

        Ok(decrypted_iter)
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
        Ok(())
    }

    pub fn generate_password(&self)-> String{
        use passwords::PasswordGenerator;
        let pg = PasswordGenerator::new()
            .length(16)
            .numbers(true)
            .lowercase_letters(true)
            .uppercase_letters(true)
            .symbols(true)
            .spaces(true)
            .exclude_similar_characters(true)
            .strict(true);
        
        pg.generate_one().unwrap()
    }
    
}


impl MasterArea {
    pub fn new() -> Self {
        let key = Key::retrieve_key().hex;
        MasterArea {
            key,
            password: String::new(),
            passed: false,
            created:false,
        }
    }

    pub fn add_master(&self, password: &str) -> Result<(), rusqlite::Error> {
        let connection = Connection::open("C:\\security_simple\\data")?;

        let encrypted_password = encryption::encrypt(password, &self.key);
        let encrypted_password_hex = hex::encode(encrypted_password);
        let mut db =
            connection.prepare("INSERT INTO master_password (id, password) VALUES (?1, ?2);")?;
        db.execute(rusqlite::params![1, encrypted_password_hex])?;
        Ok(())
    }

    fn check_master<'a>(&'a self, master_password: &'a str) -> Result<&str, rusqlite::Error> {
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

}
