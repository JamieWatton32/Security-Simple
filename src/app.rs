use egui::{FontId, Response, RichText};
use rusqlite::{Connection, Error as SqErr, Result};
use crate::encryption::*;

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct Database {
    master: String,
    key: Vec<u8>,
    site: String,
    password: String,
    user_data: Vec<User>,
    extracted_master: String,
    passed_master_check: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct User {
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

impl Database {
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            key,
            passed_master_check: false,
            ..Default::default()
        }
    }

    pub fn add(&self, site_name: &str, password: &str) -> Result<(), DbErr> {
        let connection = Connection::open("C:/Users/jamie/passwords/data")?;
        let encrypted_password = encryption::encrypt(password, &self.key);
        let encrypted_password_hex = hex::encode(encrypted_password);
        let mut db =
            connection.prepare("INSERT INTO passwords (site, password) VALUES (?1, ?2);")?;
        db.execute(rusqlite::params![site_name, encrypted_password_hex])?;
        Ok(())
    }

    fn fetch_from_db(&mut self) -> Result<()> {
        let conn = Connection::open("C:/Users/jamie/passwords/data")?;
        let mut stmt = conn.prepare("SELECT site, password FROM passwords")?;
        let user_iter = stmt.query_map([], |row| {
            let encrypted_password_hex: String = row.get(1)?;
            let encrypted_password = hex::decode(encrypted_password_hex).unwrap();
            let decrypted_password = encryption::decrypt(&encrypted_password, &self.key);
            Ok(User {
                site: row.get(0)?,
                password: decrypted_password,
            })
        })?;

        self.user_data.clear();
        for user in user_iter {
            match user {
                Ok(u) => self.user_data.push(u),
                Err(e) => eprintln!("Error reading row: {:?}", e),
            }
        }

        Ok(())
    }

    pub fn add_master(&self, password: &str) -> Result<(), DbErr> {
        let connection = Connection::open("C:/Users/jamie/passwords/data")?;
        let encrypted_password = encryption::encrypt(password, &self.key);
        let encrypted_password_hex = hex::encode(encrypted_password);
        let mut db =
            connection.prepare("INSERT INTO master_password (id, password) VALUES (?1, ?2);")?;
        db.execute(rusqlite::params![1, encrypted_password_hex])?;
        Ok(())
    }

    fn check_master<'a>(&'a self, master_password: &'a str, key: &Vec<u8>) -> Result<&str> {
        let connection = Connection::open("C:/Users/jamie/passwords/data")?;
        let mut db = connection.prepare("SELECT password FROM master_password Where id = (?1);")?;
        let mut rows = db.query(rusqlite::params![1])?;

        if let Ok(Some(row)) = rows.next() {
            let encrypted_password_hex: String = row.get(0)?;
            let encrypted_password = hex::decode(encrypted_password_hex).unwrap();
            let decrypted_password = encryption::decrypt(&encrypted_password, &key);
            if decrypted_password == master_password {
                Ok(master_password)
            } else {
                Ok("Passwords did not match.")
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
            ui.heading("User Entry Application");

            ui.label(RichText::new("Enter master password").font(FontId::proportional(40.0)));
            let response: Response = ui
                .add(egui::TextEdit::singleline(&mut self.master).font(FontId::proportional(20.0)));

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.extracted_master = self.extract_master();
                if &self.extracted_master[..] == self.master {
                    self.passed_master_check = true;
                }
                self.master.clear();
            }

            ui.horizontal(|ui| {
                ui.heading("Enter Site name");
                ui.text_edit_singleline(&mut self.site);
            });

            ui.horizontal(|ui| {
                ui.heading("Enter password for the above site");
                ui.text_edit_singleline(&mut self.password);
            });

            if ui.button("Add to database").clicked() {
                if let Err(err) = self.add(&self.site, &self.password) {
                    eprintln!("Error saving to database: {:?}", err);
                }
            }

            ui.separator();
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
