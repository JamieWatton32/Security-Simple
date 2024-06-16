use egui::{FontId, Response, RichText};

use crate::{
    areas::{MasterArea, RegularArea},
    key::Key,
};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Database {
    key: Key,
    master: MasterArea,
    regular: RegularArea,
    inputs: Inputs,
}
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Inputs {
    site_current: String,
    password_current: String,
}

impl Inputs {
    fn empty() -> Inputs {
        Inputs {
            site_current: String::new(),
            password_current: String::new(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Decrypt {
    pub site: String,
    pub password: String,
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
            let test_password: &str = "test123";
            let _ = self.master.add_master(test_password);
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
                    ui.text_edit_singleline(&mut self.inputs.password_current)
                        .enabled();
                });

                if ui.button("Add to database").clicked() {
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
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    for site in &self.regular.sites {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Site Name: {}", site))
                                    .font(FontId::proportional(20.0)),
                            );

                            
                        });
                    }
                    for pw in &self.regular.passwords {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Site Name: {}", pw))
                                    .font(FontId::proportional(20.0)),
                            );

                            
                        });
                    }
                });
            }
        });
     
    }
}
