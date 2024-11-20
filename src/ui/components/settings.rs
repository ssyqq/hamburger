use crate::ui::state::{SettingsState, UIState};
use eframe::egui::{self, Ui};

#[derive(Default)]
pub struct Settings {
    temp_settings: SettingsState,
}

impl Settings {
    pub fn ui(&mut self, ui: &mut Ui, state: &mut UIState) {
        ui.vertical(|ui| {
            ui.heading("Settings");

            ui.group(|ui| {
                ui.label("API Configuration");

                ui.horizontal(|ui| {
                    ui.label("API Key:");
                    ui.text_edit_singleline(&mut self.temp_settings.api_key);
                });

                ui.horizontal(|ui| {
                    ui.label("API Base URL:");
                    ui.text_edit_singleline(&mut self.temp_settings.api_base);
                });
            });

            ui.group(|ui| {
                ui.label("Model Configuration");

                ui.horizontal(|ui| {
                    ui.label("Model:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.temp_settings.model)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.temp_settings.model,
                                "gpt-3.5-turbo".to_string(),
                                "GPT-3.5 Turbo",
                            );
                            ui.selectable_value(
                                &mut self.temp_settings.model,
                                "gpt-4".to_string(),
                                "GPT-4",
                            );
                            ui.selectable_value(
                                &mut self.temp_settings.model,
                                "mistral-large-latest".to_string(),
                                "Mistral Large",
                            );
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Temperature:");
                    ui.add(
                        egui::Slider::new(&mut self.temp_settings.temperature, 0.0..=2.0)
                            .text("temperature"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Max Tokens:");
                    ui.add(
                        egui::Slider::new(&mut self.temp_settings.max_tokens, 100..=4000)
                            .text("max tokens"),
                    );
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    // 保存设置
                    state.settings = self.temp_settings.clone();
                    state.show_settings = false;
                }

                if ui.button("Cancel").clicked() {
                    // 取消修改
                    self.temp_settings = state.settings.clone();
                    state.show_settings = false;
                }
            });
        });
    }
}
