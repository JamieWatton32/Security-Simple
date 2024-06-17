use egui::{FontId, Response, RichText};
use rusqlite::types::FromSqlError;

use crate::{
    areas::{DbErr, MasterArea, RegularArea},
    key::Key,
};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Database {
    key: Key,
    master: MasterArea,
    regular: RegularArea,
    inputs: Inputs,
}

//For keepign track of the current string inside of text boxs,
//needed because e
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Inputs {
    site_current: String,
    password_current: String,
    master_current: String,
}
impl Inputs {
    fn empty() -> Inputs {
        Inputs {
            site_current: String::new(),
            password_current: String::new(),
            master_current: String::new(),
        }
    }
}

impl Database {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            master: MasterArea::new(),
            regular: RegularArea::new(),
            inputs: Inputs::empty(),
        }
    }
}

impl eframe::App for Database {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.master.created {
                ui.label(
                    RichText::new("Please create masterword.").font(FontId::proportional(40.0)),
                );
                ui.label(
                    RichText::new("If lost there is no way to recover this!")
                        .font(FontId::proportional(40.0)),
                );
                let response_mp: Response = ui.add(
                    egui::TextEdit::singleline(&mut self.master.password)
                        .font(FontId::proportional(20.0)),
                );
                if response_mp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    match self.master.add_master(&self.master.password) {
                        Ok(_) => {
                            ui.label(
                                RichText::new("Created succesfully!")
                                    .font(FontId::proportional(40.0)),
                            );
                            self.master.created = true;
                        }
                        Err(_) => {
                            ui.label(
                                RichText::new("Incorrect password. Try again!")
                                    .font(FontId::proportional(40.0)),
                            );
                        }
                    }
                }
            }

            if !self.master.passed && self.master.created {
                ui.label(RichText::new("Enter master password").font(FontId::proportional(40.0)));
                let response: Response = ui.add(egui::TextEdit::password(
                    egui::TextEdit::singleline(&mut self.master.password)
                        .font(FontId::proportional(20.0)),
                    true,
                ));

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if self.master.extract_master() == self.master.password {
                        self.master.passed = true;
                    }
                }
            }

            ui.separator();
            //do not enable site/pw entry until mastpass is passed
            if self.master.passed {
                ui.is_enabled();

                ui.horizontal(|ui| {
                    ui.heading("Enter Site name");
                    ui.text_edit_singleline(&mut self.inputs.site_current);
                });

                ui.horizontal(|ui| {
                    ui.heading("Enter password for the above site");
                    if ui
                        .button(
                            RichText::new("Generate random password?")
                                .font(FontId::proportional(10.0)),
                        )
                        .clicked()
                    {
                        self.inputs.password_current = self.regular.generate_password()
                            + "-"
                            + &self.regular.generate_password();
                    } else {
                        ui.text_edit_singleline(&mut self.inputs.password_current);
                    }
                });

                if ui
                    .button(RichText::new("Add password!").font(FontId::proportional(20.0)))
                    .clicked()
                {
                    if let Err(err) = self
                        .regular
                        .add_regular(&self.inputs.site_current, &self.inputs.password_current)
                    {
                        eprintln!("Error saving to database: {:?}", err);
                    }
                }
            }

            ui.separator();
            self.regular.sites.clear();
            self.regular.passwords.clear();
            if self.master.passed {
                if let Err(err) = self.regular.fetch_from_db() {
                    eprintln!("Error fetching from database: {}", err);
                }

                ui.heading("Stored Data");
                ui.separator();
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    for (idx, pw) in self.regular.passwords.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.separator();
                            ui.label(
                                RichText::new(format!(
                                    "Password for {}: {}",
                                    self.regular.sites[idx], pw
                                ))
                                .font(FontId::proportional(20.0)),
                            );
                        });
                    }
                });
            }
        });
    }
}
