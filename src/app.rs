use std::{f32::consts::PI, path::PathBuf};

use eframe::CreationContext;
use serde::{Deserialize, Serialize};

use crate::{types::DragknifeConfig, DragknifePath};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct DragknifeApp {
    config: DragknifeConfig,
    output_name: String,
    input_file: Option<PathBuf>,
    #[serde(skip)]
    output_contents: Option<String>,
}

impl Default for DragknifeApp {
    fn default() -> Self {
        Self {
            config: DragknifeConfig {
                knife_offset: 1.,
                swivel_lift_height: 1.,
                sharp_angle_threshold: 10. * PI / 180.,
            },
            input_file: None,
            output_contents: None,
            output_name: "".to_string(),
        }
    }
}

impl DragknifeApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }
}

impl eframe::App for DragknifeApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            config,
            input_file,
            output_name,
            output_contents,
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Dragknife settings");
            egui::warn_if_debug_build(ui);

            ui.add(
                egui::Slider::new(&mut config.knife_offset, 0.0..=50.0)
                    .text("Dragknife offset (mm)"),
            );
            ui.add(
                egui::Slider::new(&mut config.swivel_lift_height, 0.0..=50.0)
                    .text("Swivel lift height (mm)"),
            );
            ui.add(
                egui::Slider::from_get_set(0.0..=180.0, |optional| {
                    if let Some(v) = optional {
                        config.sharp_angle_threshold = v as f32 * PI / 180.;
                    }
                    (config.sharp_angle_threshold * 180. / PI).into()
                })
                .text("Sharp corner threshold (°)"),
            );
            ui.separator();
            ui.add(egui::TextEdit::singleline(output_name).hint_text("Output filename"));
            if ui.button("Open file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    *input_file = Some(path);
                }
            }
            if let Some(picked_path) = input_file {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    egui::ScrollArea::horizontal()
                        .stick_to_right(true)
                        .show(ui, |ui| ui.monospace(picked_path.display().to_string()));
                });
                if ui.button("Repath").clicked() {
                    let fc = std::fs::read_to_string(&picked_path).unwrap();
                    let got: Vec<_> = gcode::parse(&fc).collect();
                    let path = DragknifePath::from_gcode(got.iter());
                    let fixed = path.to_fixed_gcode(&config);
                    let output_file = picked_path.with_file_name(output_name);
                    let output = fixed.iter().map(|g| format!("{}\n", g)).collect::<String>();
                    std::fs::write(output_file, &output).unwrap();
                    *output_contents = Some(output);
                }
            }
            ui.collapsing("Repathed GCode", |ui| {
                if let Some(output) = output_contents {
                    if ui.button("📋").on_hover_text("Click to copy").clicked() {
                        ui.output_mut(|o| o.copied_text = output.clone());
                    }
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(output.as_str());
                    });
                } else {
                    ui.label("No output");
                }
            });
        });
    }
}
