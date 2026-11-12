#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use chrono::{DateTime, FixedOffset, Utc};
use clipboard_win::{Clipboard, Getter, Setter, formats};
use dateparser::timezone;
use eframe::egui;
use egui::Color32;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Timestamp Xlate",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    first_render: bool,
    ui_ctx: egui::Context,
    source_tz_str: String,
    source_tz: FixedOffset,
    timestamp_str: String,
    timezones_str: String,
    datetime: DateTime<Utc>,
    errors: Vec<String>,
}

impl eframe::App for MyApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui_ctx = ctx.clone();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.first_render {
            // Calculate desired size
            let new_size = egui::Vec2::new(600.0, 400.0);
            // Send the command
            self.ui_ctx
                .send_viewport_cmd(egui::ViewportCommand::InnerSize(new_size));
            self.first_render = false;
        }
        egui::CentralPanel::default().show(ui, |ui| {
            self.show_main_content(ui);
        });
        // let size = self.ui_ctx.viewport_rect().size();
        // self.ui_ctx
        //     .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
    }
}

impl MyApp {
    fn new(cc: &eframe::CreationContext) -> Self {
        let utcnow = Utc::now();
        Self {
            first_render: true,
            ui_ctx: cc.egui_ctx.clone(),
            source_tz_str: String::from("UTC"),
            source_tz: FixedOffset::west_opt(0).unwrap(),
            timestamp_str: utcnow.to_string(),
            timezones_str: String::from("PST, CST, EDT"),
            datetime: utcnow,
            errors: Vec::<String>::new(),
        }
    }

    fn show_main_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("Timestamp Xlate");
        egui::Grid::new("main_widget_grid")
            .min_col_width(100.0)
            .max_col_width(400.0)
            .show(ui, |ui| {
                let name_label_widget =
                    ui.add(egui::Label::new("Source TZ").halign(egui::Align::Max));
                ui.text_edit_singleline(&mut self.source_tz_str)
                    .labelled_by(name_label_widget.id);
                ui.end_row();
                let name_label = ui.add(egui::Label::new("Timestamp").halign(egui::Align::Max));
                ui.text_edit_singleline(&mut self.timestamp_str)
                    .labelled_by(name_label.id);
                ui.end_row();
                let timezones_label =
                    ui.add(egui::Label::new("Timezones").halign(egui::Align::Max));
                ui.text_edit_singleline(&mut self.timezones_str)
                    .labelled_by(timezones_label.id);
                ui.end_row();
            });
        if let Some(source_tz) = timezone::parse(&self.source_tz_str).ok() {
            self.source_tz = source_tz;
        } else {
            self.errors.push(format!(
                "Failed to parse source TZ '{}'",
                self.source_tz_str
            ));
        }
        if let Some(datetime) =
            dateparser::parse_with_timezone(self.timestamp_str.as_str(), &self.source_tz).ok()
        {
            self.datetime = datetime;
        } else {
            self.errors.push(format!(
                "Failed to parse source timestamp '{}' with timezone '{}'",
                self.timestamp_str, self.source_tz
            ))
        }
        egui::Grid::new("timezone_disp_grid")
            .min_col_width(100.0)
            .max_col_width(400.0)
            .show(ui, |ui| {
                let tzs: Vec<FixedOffset> = self
                    .timezones_str
                    .clone()
                    .replace(" ", "")
                    .split(",")
                    .filter_map(|tzs| {
                        if let Some(tz) = timezone::parse(tzs).ok() {
                            return Some(tz);
                        } else {
                            self.errors
                                .push(format!("Failed to parse TZ '{}'", tzs.to_string()));
                            return None;
                        }
                    })
                    .collect();
                for tz in tzs {
                    let localized_datetime_str = self.datetime.with_timezone(&tz).to_string();
                    ui.label(tz.to_string());
                    ui.label(localized_datetime_str.clone());
                    if ui.button("Copy").clicked() {
                        if let Some(cb) = Clipboard::new_attempts(10).ok() {
                            let lds = localized_datetime_str.clone();
                            match formats::Unicode.write_clipboard(&lds.as_str()) {
                                Ok(_) => {}
                                Err(_) => self.errors.push(String::from("Failed to copy")),
                            }
                        } else {
                            self.errors.push(String::from("Failed to copy"));
                        }
                    }
                    ui.end_row();
                }
            });
        ui.vertical(|ui| {
            for err in self.errors.drain(..) {
                ui.label(egui::RichText::new(err).color(Color32::LIGHT_RED));
            }
        });
    }
}
