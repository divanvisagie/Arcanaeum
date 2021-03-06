use std::collections::{HashMap, HashSet};

use eframe::egui::Color32;
use eframe::egui::{self};
use eframe::epi;
use egui::Ui;

use crate::parser::SaveInfo;
use crate::sktypes::skui_value::{SkUIValue, UIValueType};
use crate::sktypes::types::SkTypeReadable;
use crate::{load_installed, load_mod_map, sktypes};
use crate::{load_save_file, mod_search::vortex_scanner::Plugin};

pub struct AppState {
    pub file_path: String,
    pub save_info: Option<SaveInfo>,
    pub mod_map: HashMap<String, Plugin>,
    pub installed: HashSet<String>,
    pub error: Option<String>,
    pub plugins: Option<Vec<SkUIValue>>,
}

fn label_line(ui: &mut Ui, name: &str, value: &str) {
    ui.label(name);
    ui.label(value);
    ui.end_row();
}

impl epi::App for AppState {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
            if ui.button("Browse to file").clicked() {
                let res = rfd::FileDialog::new()
                    .add_filter("Elder Scrolls Save", &["ess"])
                    .set_directory("./input")
                    .pick_file();

                match res {
                    Some(path_buf) => {
                        self.file_path = String::from(path_buf.to_str().unwrap());
                        match load_save_file(self.file_path.to_string()) {
                            Ok(values) => {
                                if values.header.is_se {
                                    self.mod_map = load_mod_map("skyrimse");
                                    self.installed = load_installed("skyrimse")
                                } else {
                                    self.mod_map = load_mod_map("skyrim");
                                    self.installed = load_installed("skyrim");
                                }
                                self.error = None;

                                let mut plugins = Vec::new();
                                for plugin_name in &values.plugin_info.plugins {
                                    let new_plugin = SkUIValue::new(
                                        plugin_name.as_str(),
                                        plugin_name.to_string(),
                                        UIValueType::Plugin,
                                    );
                                    plugins.push(new_plugin);
                                }

                                self.plugins = Some(plugins);
                                self.save_info = Some(values);
                            }
                            Err(e) => {
                                self.error = Some(e.to_string());
                                self.save_info = None;
                                self.plugins = None;
                            }
                        };
                    }
                    None => tracing::error!("No file selected"),
                }
            }
            if let Some(e) = &self.error {
                ui.colored_label(Color32::from_rgb(200, 50, 50), e);
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::CentralPanel::default().show(ctx, |_ui| {
                egui::Grid::new("values")
                    .striped(true)
                    .min_row_height(22.)
                    .min_col_width(400.0)
                    .show(ui, |ui| {
                        if let Some(si) = &self.save_info {
                            if si.header.is_se {
                                label_line(ui, "Game", "Skyrim Special Edition");
                            } else {
                                label_line(ui, "Game", "Skyrim");
                            }

                            label_line(
                                ui,
                                "Save Number",
                                si.header.save_number.to_string().as_str(),
                            );

                            label_line(ui, "Character Name", si.header.player_name.as_str());
                            label_line(
                                ui,
                                "Character Level",
                                si.header.player_level.to_string().as_str(),
                            );
                            label_line(
                                ui,
                                "Character Sex",
                                si.header.player_sex.to_string().as_str(),
                            );
                            label_line(
                                ui,
                                "Character Race",
                                si.header.player_race_editor_id.as_str(),
                            );
                            label_line(ui, "In Game Date", si.header.game_date.as_str());
                            label_line(ui, "Player Location", si.header.player_location.as_str());
                        }
                    });
            });
            if let Some(_plugins) = &self.plugins {
                ui.separator();
                ui.heading("Plugins");
                ui.separator();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("values")
                    .striped(true)
                    .min_row_height(22.)
                    .min_col_width(400.0)
                    .max_col_width(400.0)
                    .show(ui, |ui| {
                        if let Some(plugins) = &self.plugins {
                            for value_entry in plugins {
                                ui.label(value_entry.get_name());
                                match value_entry.plugin_type {
                                    sktypes::skui_value::PluginType::Native => {
                                        ui.label("Original Game File/DLC");
                                    }
                                    sktypes::skui_value::PluginType::CreationClub => {
                                        ui.label("Creation Club Mod");
                                    }
                                    sktypes::skui_value::PluginType::Mod => {
                                        let key = &value_entry.get_value_string();
                                        if self.installed.contains(key) {
                                            ui.colored_label(
                                                Color32::from_rgb(50, 200, 50),
                                                "Installed",
                                            );
                                        } else if self.mod_map.contains_key(key) {
                                            let value = self.mod_map.get(key).unwrap();

                                            egui::ScrollArea::vertical().show(ui, |ui| {
                                                for l in value.urls.clone() {
                                                    ui.hyperlink(l.as_str());
                                                    ui.end_row();
                                                }
                                            });
                                        } else {
                                            ui.colored_label(
                                                Color32::from_rgb(200, 50, 50),
                                                "Not Found",
                                            );
                                        }
                                    }
                                    sktypes::skui_value::PluginType::NotAPlugin => {}
                                }
                                ui.end_row();
                            }
                        }
                    });
            });
        });
    }

    fn name(&self) -> &str {
        "Arcanaeum"
    }
}
