use std::fs::File;
use std::io::{prelude::*, Result};
use std::{f32::consts::PI, path::PathBuf};

use eframe::CreationContext;
use gcode::Mnemonic;
use serde::{Deserialize, Serialize};

use crate::types::LiftConfig;
use crate::{types::DragknifeConfig, DragknifePath};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct DragknifeApp {
    config: DragknifeConfig,
    output_name: String,
    input_file: Option<PathBuf>,
    skip_m3: bool,
    #[serde(skip)]
    output_contents: Result<Option<String>>,
    #[serde(skip)]
    output_file: Option<PathBuf>,
}

impl Default for DragknifeApp {
    fn default() -> Self {
        Self {
            config: DragknifeConfig {
                knife_offset: 1.,
                lift_config: LiftConfig::RelativeHeight(1.0),
                sharp_angle_threshold: 10. * PI / 180.,
                swivel_feedrate: 300.,
            },
            input_file: None,
            output_file: None,
            skip_m3: true,
            output_contents: Ok(None),
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
            output_file,
            skip_m3,
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
            let selected_height = *config.lift_config.get_height_mut();
            ui.horizontal(|ui| {
                ui.label("Swivel lift type");
                ui.selectable_value(
                    &mut config.lift_config,
                    LiftConfig::RelativeHeight(selected_height),
                    "Relative",
                );
                ui.selectable_value(
                    &mut config.lift_config,
                    LiftConfig::AbsoluteHeight(selected_height),
                    "Absolute",
                );
            });
            ui.add(
                egui::Slider::new(config.lift_config.get_height_mut(), 0.0..=50.0)
                    .text("Swivel lift height (mm)"),
            );
            ui.add(
                egui::Slider::new(&mut config.swivel_feedrate, 0.0..=2000.0)
                    .text("Swivel feedrate (mm/min)"),
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
            ui.checkbox(skip_m3, "Remove M3 commands from output");
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
                    match repath_and_write(picked_path, &config, &output_name, *skip_m3) {
                        Ok((output, output_file_opt)) => {
                            *output_contents = Ok(Some(output));
                            *output_file = output_file_opt
                        }
                        Err(err) => *output_contents = Err(err),
                    }
                }
            }
            if let Ok(Some(output)) = output_contents {
                ui.horizontal(|ui| {
                    if ui.button("📋").on_hover_text("Click to copy").clicked() {
                        ui.output_mut(|o| o.copied_text = output.clone());
                    }
                    if let Some(output_file_actual) = output_file {
                        egui::ScrollArea::horizontal()
                            .stick_to_right(true)
                            .id_source("second scroll area")
                            .show(ui, |ui| {
                                ui.monospace(output_file_actual.display().to_string())
                            });
                    } else {
                        ui.label("Output file name was empty: did not write to file.");
                    }
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(output.as_str());
                });
            } else if let Err(e) = output_contents {
                ui.label(format!("{e}"));
            } else {
                ui.label("No output");
            }
        });
    }
}

fn repath_and_write(
    input_file: &PathBuf,
    config: &DragknifeConfig,
    output_name: &str,
    skip_m3: bool,
) -> Result<(String, Option<PathBuf>)> {
    let fc = std::fs::read_to_string(input_file)?;
    let got: Vec<_> = gcode::parse(&fc).collect();
    let path = DragknifePath::from_gcode(got.iter());
    let fixed = path.to_fixed_gcode(config);
    let output = fixed
        .iter()
        .filter(|g| {
            !skip_m3
                || match (g.mnemonic(), g.major_number()) {
                    (Mnemonic::Miscellaneous, 3) => false,
                    _ => true,
                }
        })
        .map(|g| format!("{}\n", g))
        .collect::<String>();
    let output_file = if !output_name.is_empty() {
        let output_file = input_file.with_file_name(output_name);
        let file = File::create(&output_file)?;
        write!(&file, "{output}")?;
        Some(output_file)
    } else {
        None
    };
    Ok((output, output_file))
}
