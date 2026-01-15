// Spectral Compressor: an FFT based compressor
// Copyright (C) 2021-2024 Robbert van der Helm
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use atomic_float::AtomicF32;
use crossbeam::atomic::AtomicCell;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::analyzer::AnalyzerData;
use crate::SpectralCompressorParams;

mod analyzer;

/// The entire GUI's width when expanded (with analyzer visible), in logical pixels.
const EXPANDED_GUI_WIDTH: f32 = 1360.0;
/// The width of the GUI's main part containing the controls (without analyzer).
const COLLAPSED_GUI_WIDTH: f32 = 680.0;
/// The entire GUI's height, in logical pixels.
const GUI_HEIGHT: f32 = 530.0;

/// Column width for the parameter sections
const COLUMN_WIDTH: f32 = 330.0;

/// The editor's mode. Essentially just a boolean to indicate whether the analyzer is shown or not.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMode {
    #[serde(rename = "collapsed")]
    Collapsed,
    #[default]
    #[serde(rename = "analyzer-visible")]
    AnalyzerVisible,
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state(editor_mode: Arc<AtomicCell<EditorMode>>) -> Arc<EguiState> {
    EguiState::from_size(
        match editor_mode.load() {
            EditorMode::Collapsed => COLLAPSED_GUI_WIDTH as u32,
            EditorMode::AnalyzerVisible => EXPANDED_GUI_WIDTH as u32,
        },
        GUI_HEIGHT as u32,
    )
}

pub(crate) fn create(
    params: Arc<SpectralCompressorParams>,
    editor_state: Arc<EguiState>,
    editor_mode: Arc<AtomicCell<EditorMode>>,
    analyzer_data: Arc<Mutex<triple_buffer::Output<AnalyzerData>>>,
    sample_rate: Arc<AtomicF32>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        editor_state,
        (editor_mode.clone(), analyzer_data.clone(), sample_rate),
        |_, _| {},
        move |egui_ctx, setter, (editor_mode, analyzer_data, sample_rate)| {
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    // Main controls column(s)
                    ui.vertical(|ui| {
                        ui.set_width(COLLAPSED_GUI_WIDTH);
                        
                        // Mode button at the top
                        ui.horizontal(|ui| {
                            let analyzer_visible = editor_mode.load() == EditorMode::AnalyzerVisible;
                            if ui.button(if analyzer_visible {
                                "Hide analyzer"
                            } else {
                                "Show analyzer"
                            }).clicked() {
                                let new_mode = if analyzer_visible {
                                    EditorMode::Collapsed
                                } else {
                                    EditorMode::AnalyzerVisible
                                };
                                editor_mode.store(new_mode);
                                // TODO: Resize window when mode changes
                            }
                        });

                        ui.add_space(10.0);

                        // Two-column layout for parameters
                        ui.horizontal(|ui| {
                            // Left column: Globals and Threshold
                            ui.vertical(|ui| {
                                ui.set_width(COLUMN_WIDTH);
                                
                                // Globals section
                                ui.heading("Globals");
                                ui.add(widgets::ParamSlider::for_param(&params.global.output_gain, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.global.dry_wet_ratio, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.global.window_size_order, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.global.overlap_times_order, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.global.compressor_attack_ms, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.global.compressor_release_ms, setter));

                                ui.add_space(20.0);

                                // Threshold section
                                ui.heading("Threshold");
                                ui.add(widgets::ParamSlider::for_param(&params.threshold.threshold_db, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.threshold.center_frequency, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.threshold.curve_slope, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.threshold.curve_curve, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.threshold.mode, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.threshold.sc_channel_link, setter));

                                ui.label("Parameter ranges and overall gain staging are still subject to\nchange. If you use this in a project, make sure to bounce\nthings to audio just in case they'll sound different later.");
                            });

                            ui.add_space(20.0);

                            // Right column: Upwards and Downwards
                            ui.vertical(|ui| {
                                ui.set_width(COLUMN_WIDTH);
                                
                                // Upwards section
                                ui.heading("Upwards");
                                ui.add(widgets::ParamSlider::for_param(&params.compressors.upwards.threshold_offset_db, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.compressors.upwards.ratio, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.compressors.upwards.high_freq_ratio_rolloff, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.compressors.upwards.knee_width_db, setter));

                                ui.add_space(20.0);

                                // Downwards section  
                                ui.heading("Downwards");
                                ui.add(widgets::ParamSlider::for_param(&params.compressors.downwards.threshold_offset_db, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.compressors.downwards.ratio, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.compressors.downwards.high_freq_ratio_rolloff, setter));
                                ui.add(widgets::ParamSlider::for_param(&params.compressors.downwards.knee_width_db, setter));
                            });
                        });
                    });

                    // Analyzer panel (only if visible)
                    if editor_mode.load() == EditorMode::AnalyzerVisible {
                        ui.separator();
                        
                        // Analyzer widget
                        analyzer::Analyzer::new(
                            analyzer_data.clone(),
                            sample_rate.clone(),
                            params.clone(),
                        )
                        .show(ui);
                    }
                });
            });
        },
    )
}
