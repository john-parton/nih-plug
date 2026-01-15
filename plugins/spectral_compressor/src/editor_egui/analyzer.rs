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
use nih_plug::nih_debug_assert;
use nih_plug_egui::egui;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::analyzer::AnalyzerData;
use crate::curve::Curve;
use crate::SpectralCompressorParams;

// Frequency range for display
const LN_FREQ_RANGE_START_HZ: f32 = 3.4011974; // ln(30.0)
const LN_FREQ_RANGE_END_HZ: f32 = 9.998797; // ln(22000.0)
const LN_FREQ_RANGE: f32 = LN_FREQ_RANGE_END_HZ - LN_FREQ_RANGE_START_HZ;

/// The analyzer width in logical pixels
const ANALYZER_WIDTH: f32 = 660.0;
/// The analyzer height in logical pixels
const ANALYZER_HEIGHT: f32 = 510.0;

pub struct Analyzer {
    analyzer_data: Arc<Mutex<triple_buffer::Output<AnalyzerData>>>,
    sample_rate: Arc<AtomicF32>,
    _params: Arc<SpectralCompressorParams>,
}

impl Analyzer {
    pub fn new(
        analyzer_data: Arc<Mutex<triple_buffer::Output<AnalyzerData>>>,
        sample_rate: Arc<AtomicF32>,
        params: Arc<SpectralCompressorParams>,
    ) -> Self {
        Self {
            analyzer_data,
            sample_rate,
            _params: params,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(
            egui::vec2(ANALYZER_WIDTH, ANALYZER_HEIGHT),
            egui::Sense::hover(),
        );

        let rect = response.rect;
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }

        // Draw background
        painter.rect_filled(
            rect,
            0.0,
            ui.style().visuals.extreme_bg_color,
        );

        // Get the analyzer data
        let mut analyzer_data = self.analyzer_data.lock().unwrap();
        let analyzer_data = analyzer_data.read();
        let nyquist_hz = self.sample_rate.load(Ordering::Relaxed) / 2.0;

        // Draw the spectrum, threshold curves, and gain reduction
        self.draw_spectrum(ui, &painter, rect, analyzer_data, nyquist_hz);
        self.draw_threshold_curves(ui, &painter, rect, analyzer_data);
        self.draw_gain_reduction(ui, &painter, rect, analyzer_data, nyquist_hz);

        // Draw border
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color),
            egui::StrokeKind::Outside,
        );
    }

    fn draw_spectrum(
        &self,
        ui: &egui::Ui,
        painter: &egui::Painter,
        rect: egui::Rect,
        analyzer_data: &AnalyzerData,
        nyquist_hz: f32,
    ) {
        let line_width = 1.5;
        let text_color = ui.style().visuals.text_color();

        // Draw vertical bars for the spectrum
        for (bin_idx, envelope) in analyzer_data.envelope_followers[..analyzer_data.num_bins].iter().enumerate() {
            let frequency_hz = (bin_idx as f32 / analyzer_data.num_bins as f32) * nyquist_hz;
            let t = Self::frequency_to_x_coord(frequency_hz, rect.width());
            if !(0.0..=1.0).contains(&t) {
                continue;
            }

            // Convert envelope to dB and map to height
            nih_debug_assert!(*envelope >= 0.0);
            let envelope_db = nih_plug::util::gain_to_db(*envelope);
            let height = Self::db_to_unclamped_t(envelope_db).clamp(0.0, 1.0);

            let x = rect.left() + (rect.width() * t);
            let y_top = rect.top() + (rect.height() * (1.0 - height));
            let y_bottom = rect.bottom();

            painter.line_segment(
                [egui::pos2(x, y_top), egui::pos2(x, y_bottom)],
                egui::Stroke::new(line_width, text_color),
            );
        }
    }

    fn draw_threshold_curves(
        &self,
        _ui: &egui::Ui,
        painter: &egui::Painter,
        rect: egui::Rect,
        analyzer_data: &AnalyzerData,
    ) {
        // Draw threshold curves as lines
        let downwards_color = egui::Color32::from_rgba_unmultiplied(115, 140, 153, 230);
        let upwards_color = egui::Color32::from_rgba_unmultiplied(140, 178, 165, 230);

        self.draw_threshold_curve(painter, rect, analyzer_data, true, downwards_color);
        self.draw_threshold_curve(painter, rect, analyzer_data, false, upwards_color);
    }

    fn draw_threshold_curve(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        analyzer_data: &AnalyzerData,
        is_downwards: bool,
        color: egui::Color32,
    ) {
        let curve = Curve::new(&analyzer_data.curve_params);
        let threshold_db_offset = if is_downwards {
            analyzer_data.curve_offsets_db.1  // downwards
        } else {
            analyzer_data.curve_offsets_db.0  // upwards
        };

        let mut points = Vec::new();
        let num_samples = rect.width() as usize;
        
        for i in 0..num_samples {
            let t = i as f32 / (num_samples - 1) as f32;
            let frequency_hz = Self::x_coord_to_frequency(t, rect.width());
            let threshold_db = curve.evaluate_linear(frequency_hz) + threshold_db_offset;
            let y_t = Self::db_to_unclamped_t(threshold_db).clamp(0.0, 1.0);

            let x = rect.left() + (rect.width() * t);
            let y = rect.top() + (rect.height() * (1.0 - y_t));
            points.push(egui::pos2(x, y));
        }

        painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, color)));
    }

    fn draw_gain_reduction(
        &self,
        _ui: &egui::Ui,
        painter: &egui::Painter,
        rect: egui::Rect,
        analyzer_data: &AnalyzerData,
        nyquist_hz: f32,
    ) {
        let gr_color = egui::Color32::from_rgba_unmultiplied(217, 242, 255, 204);
        let line_width = 1.5;

        // Draw gain reduction as vertical bars from the bottom
        for (bin_idx, gain_diff_db) in analyzer_data.gain_difference_db[..analyzer_data.num_bins].iter().enumerate() {
            // gain_difference_db is positive for gain, negative for attenuation
            // We want to show gain reduction (attenuation), so we visualize when it's negative
            if *gain_diff_db >= 0.0 {
                continue; // No gain reduction
            }

            let frequency_hz = (bin_idx as f32 / analyzer_data.num_bins as f32) * nyquist_hz;
            let t = Self::frequency_to_x_coord(frequency_hz, rect.width());
            if !(0.0..=1.0).contains(&t) {
                continue;
            }

            // Map gain reduction to height from bottom
            let gr_height = (gain_diff_db.abs() / 80.0).clamp(0.0, 1.0);

            let x = rect.left() + (rect.width() * t);
            let y_top = rect.bottom() - (rect.height() * gr_height);
            let y_bottom = rect.bottom();

            painter.line_segment(
                [egui::pos2(x, y_top), egui::pos2(x, y_bottom)],
                egui::Stroke::new(line_width, gr_color),
            );
        }
    }

    /// Convert a frequency to an x-coordinate (0.0-1.0) using logarithmic scaling
    fn frequency_to_x_coord(frequency_hz: f32, _width: f32) -> f32 {
        let ln_freq = frequency_hz.ln();
        ((ln_freq - LN_FREQ_RANGE_START_HZ) / LN_FREQ_RANGE).clamp(0.0, 1.0)
    }

    /// Convert an x-coordinate (0.0-1.0) back to a frequency using logarithmic scaling
    fn x_coord_to_frequency(t: f32, _width: f32) -> f32 {
        let ln_freq = LN_FREQ_RANGE_START_HZ + (t * LN_FREQ_RANGE);
        ln_freq.exp()
    }

    /// Convert a dB value to a vertical coordinate (0.0-1.0)
    /// -80 dB maps to 0.0, +20 dB maps to 1.0
    fn db_to_unclamped_t(db_value: f32) -> f32 {
        (db_value + 80.0) / 100.0
    }
}
