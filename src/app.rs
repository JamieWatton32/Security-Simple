use crate::encryption::*;
use egui::{FontId, Response, RichText};
use key::StoreKey;
use rusqlite::{Connection, Error as SqErr, Result};

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct Database {
    master: String,
    key: Vec<u8>,
    site: String,
    password: String,
    user_data: Vec<Decrypt>,
    extracted_master: String,
    passed_master_check: bool,
}
#[derive(Debug,serde::Deserialize, serde::Serialize,Default)]
enum Passwords {
    Master,
    #[default] Regular,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct Decrypt {
    pub site: String,
    pub password: String,
}
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
fn decrypt_string(hex: String, key: &[u8]) -> Result<String> {
    let encrypted_password = hex::decode(hex).unwrap();

    Ok(encryption::decrypt(&encrypted_password, &key))
}
impl Database {
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            key,
            passed_master_check: false,
            ..Default::default()
        }
    }

    pub fn add_regular(&self, site_name: &str, password: &str) -> Result<(), DbErr> {
        let connection = Connection::open("C:\\security_simple\\data")?;
        let encrypted_password = encryption::encrypt(password, &self.key);
        let encrypted_password_hex = hex::encode(encrypted_password);
        let mut db =
            connection.prepare("INSERT INTO passwords (site, password) VALUES (?1, ?2);")?;
        db.execute(rusqlite::params![site_name, encrypted_password_hex])?;
        Ok(())
    }

    fn fetch_from_db(&mut self) -> Result<()> {
        let conn = Connection::open("C:\\security_simple\\data").unwrap();
        let mut stmt = conn
            .prepare("SELECT site, password FROM passwords")
            .unwrap();
        let mut user_iter = stmt.query_map([], |row| {
            let encrypted_password_hex: String = row.get(1)?;
            if let Ok(decrypted) = decrypt_string(encrypted_password_hex, &self.key) {
                Ok(Decrypt {
                    site: row.get(0)?,
                    password: decrypted,
                })
            } else {
                Err(SqErr::QueryReturnedNoRows)
            }
        })?;
        for user in &mut user_iter {
            match user {
                Ok(user) => self.user_data.push(user),
                Err(error) => eprintln!("Error reading row: {:?}", error),
            }
        }

        Ok(())
    }

    pub fn add_master(&self, password: &str) -> Result<(), DbErr> {
        let connection = Connection::open("C:\\security_simple\\data")?;
        
        let encrypted_password = encryption::encrypt(password, &self.key);
        let encrypted_password_hex = hex::encode(encrypted_password);
        let mut db =
            connection.prepare("INSERT INTO master_password (id, password) VALUES (?1, ?2);")?;
        db.execute(rusqlite::params![1, encrypted_password_hex])?;
        Ok(())
    }

    fn check_master<'a>(&'a self, master_password: &'a str, key: &Vec<u8>) -> Result<&str> {
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
        if let Ok(p) = self.check_master(&self.master, &self.key) {
            p.to_owned()
        } else {
            String::from("Invalid Password!")
        }
    }
}

impl eframe::App for Database {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let _ = self.add_master("test123"); // this will attempt to add the password, but silently fails if it already exists.
            ui.heading(RichText::new("Enter master password"));

            ui.label(RichText::new("Enter master password").font(FontId::proportional(40.0)));

            let response: Response = ui.add(egui::TextEdit::password(
                egui::TextEdit::singleline(&mut self.master).font(FontId::proportional(20.0)),
                true,
            ));

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.extracted_master = self.extract_master();
                if &self.extracted_master[..] == self.master {
                    self.passed_master_check = true;
                }
            }
            ui.separator();
            //do not enable site/pw entry until mastpass is passed
            if self.passed_master_check {
                ui.is_enabled();

                ui.horizontal(|ui| {
                    ui.heading("Enter Site name");
                    ui.text_edit_singleline(&mut self.site);
                });

                ui.horizontal(|ui| {
                    ui.heading("Enter password for the above site");
                    ui.text_edit_singleline(&mut self.password).enabled();
                });

                if ui.button("Add to database").clicked() {
                    if let Err(err) = self.add_regular(&self.site, &self.password) {
                        eprintln!("Error saving to database: {:?}", err);
                    }
                }
            }

            ui.separator();
            self.user_data.clear();
            if self.passed_master_check {
                if let Err(err) = self.fetch_from_db() {
                    eprintln!("Error fetching from database: {}", err);
                }
                ui.heading("Stored Data");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for user in &self.user_data {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Site Name: {}", user.site))
                                    .font(FontId::proportional(20.0)),
                            );

                            ui.label(
                                RichText::new(format!("Password: {}", user.password))
                                    .font(FontId::proportional(20.0)),
                            );
                        });
                    }
                });
            }
        });
    }
}
