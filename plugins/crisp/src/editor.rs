// Crisp: a distortion plugin but not quite
// Copyright (C) 2022-2024 Robbert van der Helm
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

use nih_plug::prelude::Editor;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::Arc;

use crate::CrispParams;

/// The width and height of the GUI in logical pixels.
const EDITOR_WIDTH: f32 = 400.0;
const EDITOR_HEIGHT: f32 = 390.0;

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<EguiState> {
    EguiState::from_size(EDITOR_WIDTH as u32, EDITOR_HEIGHT as u32)
}

pub(crate) fn create(
    params: Arc<CrispParams>,
    editor_state: Arc<EguiState>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        editor_state,
        (),
        |_, _| {},
        move |egui_ctx, setter, _state| {
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                ui.vertical(|ui| {
                    // Title centered at the top
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.heading(
                            egui::RichText::new("Crisp")
                                .size(30.0)
                                .font(egui::FontId::proportional(30.0)),
                        );
                        ui.add_space(10.0);
                    });

                    // Scrollable area for all the parameters
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.add_space(5.0);

                            // Use the generic UI to display all parameters
                            widgets::generic_ui::create(
                                ui,
                                params.clone(),
                                setter,
                                widgets::generic_ui::GenericSlider,
                            );
                        });
                });
            });
        },
    )
}
