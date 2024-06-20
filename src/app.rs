use egui::{FontId, Response, RichText, Ui};

use crate::{key::Key, master::MasterArea, regular::RegularArea};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Database {
    key: Key,
    master: MasterArea,
    regular: RegularArea,
    inputs: Inputs,
}

//For keepign track of the current string inside of text boxs,
//needed because loops and  stuff
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Inputs {
    site_current: String,
    password_current: String,
    to_delete: Option<usize>,
    confirm: bool,
}
impl Inputs {
    fn empty() -> Inputs {
        Inputs {
            site_current: String::new(),
            password_current: String::new(),
            to_delete: None,
            confirm: false,
        }
    }
}

//This is to help make the button statements a bit cleaner. probably a bad idea but idc
fn button_text(text: &str, font_size: f32) -> RichText {
    RichText::new(text).font(FontId::proportional(font_size))
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
    fn create_master(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.label(RichText::new("Please create masterword.").font(FontId::proportional(40.0)));
            ui.label(
                RichText::new("If lost there is no way to recover this!")
                    .font(FontId::proportional(40.0)),
            );
            let response_mp: Response = ui.add(
                egui::TextEdit::singleline(&mut self.master.password)
                    .char_limit(0x18)
                    .font(FontId::proportional(20.0)),
            );
            if response_mp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                match self.master.add_master(&self.master.password) {
                    Ok(_) => {
                        ui.label(
                            RichText::new("Created succesfully!").font(FontId::proportional(40.0)),
                        );
                    }
                    Err(_) => {
                        ui.label(
                            RichText::new("Incorrect password. Try again!")
                                .font(FontId::proportional(40.0)),
                        );
                    }
                }
            }
        });
    }

    fn enter_master(&mut self, ui: &mut egui::Ui) {
        ui.label(button_text("Enter master password", 40.0));
        let response: Response = ui.add(egui::TextEdit::password(
            egui::TextEdit::singleline(&mut self.master.password)
                .char_limit(0x18)
                .font(FontId::proportional(20.0)),
            true,
        ));

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if self.master.extract_master() == self.master.password {
                self.master.passed = true;
            }
        }
    }

    fn password_entry(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Enter password for the above site");
            if ui.button(button_text("Random password?", 15.0)).clicked() {
                self.inputs.password_current =
                    self.regular.generate_password() + "-" + &self.regular.generate_password();
            } else {
                ui.text_edit_singleline(&mut self.inputs.password_current);
            }
        });
    }

    fn show_passwords(&mut self, ui: &mut Ui) {
        egui::ScrollArea::horizontal().show(ui, |ui| {
            for (idx, pw) in self.regular.passwords.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.separator();

                    ui.label(
                        RichText::new(format!("Site: {}: {}", self.regular.sites[idx], pw))
                            .font(FontId::proportional(20.0)),
                    );

                    if ui
                        .button(RichText::new("Delete?").font(FontId::proportional(15.0)))
                        .clicked()
                    {
                        self.inputs.confirm = true;
                        self.inputs.to_delete = Some(idx);
                    }
                });
            }
        });
    }
    fn delete_row(&self, idx: usize) {
        match self.regular.remove_entry(idx) {
            Ok(rows_affected) => {
                if rows_affected > 0 {
                    println!("Row with ID {} deleted.", idx);
                } else {
                    println!("No row found with ID {}.", idx);
                }
            }
            Err(e) => {
                eprintln!("Error deleting row with ID {}: {}", idx, e);
            }
        }
    }
    fn popup_confirm(&mut self, ctx: &egui::Context) {
        if let Some(idx) = self.inputs.to_delete {
            egui::Window::new("Confirmation")
                .max_height(100.0)
                .collapsible(false)
                .auto_sized()
                .resizable(false)
                .show(ctx, |ui| {
                    let text = format!(
                        "Are you sure you would like to delete the password for {}",
                        self.regular.sites[idx]
                    );
                    ui.label(button_text(&text, 20.0));
                    ui.vertical_centered(|ui| {

                        //todo: fix how the arrays work..
                        if ui.button(button_text("yes delete!", 15.0)).clicked() {
                            let idx = self.regular.ids[idx];
                            self.delete_row(idx.try_into().unwrap());
                            self.inputs.confirm = false;
                            self.inputs.to_delete = None;
                        }
                        if ui
                            .button(button_text("No, No dont delete!!", 15.0))
                            .clicked()
                        {
                            self.inputs.confirm = false;
                            self.inputs.to_delete = None;
                        }
                    });
                });
        }
    }
}

impl eframe::App for Database {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //checks if MP already exists. If not then gets user to create one.
        if !self.master.master_exists() {
            self.create_master(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            //MP entry.
            if !self.master.passed && self.master.master_exists() {
                self.enter_master(ui);
            }

            ui.separator();
            //do not enable site/pw entry until mastpass is passed
            if self.master.passed {
                ui.is_enabled();

                ui.horizontal(|ui| {
                    ui.heading("Enter Site name");
                    ui.text_edit_singleline(&mut self.inputs.site_current);
                });

                self.password_entry(ui);

                if ui.button(button_text("add password?", 20.0)).clicked() {
                    let (site_name, pass_entry) =
                        (&self.inputs.site_current, &self.inputs.password_current);
                    if let Err(err) = self.regular.add_regular(site_name, pass_entry) {
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
                self.show_passwords(ui);

                if self.inputs.confirm {
                    self.popup_confirm(ctx);
                }
            }
        });
    }
}
