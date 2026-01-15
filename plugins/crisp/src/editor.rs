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
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;

use crate::CrispParams;

#[derive(Lens)]
struct Data {
    params: Arc<CrispParams>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (400, 390))
}

pub(crate) fn create(
    params: Arc<CrispParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            // Wrapper HStack to center the title horizontally
            HStack::new(cx, |cx| {
                Element::new(cx).width(Stretch(1.0));
                Label::new(cx, "Crisp")
                    .font_family(vec![FamilyOwned::Named(String::from(assets::NOTO_SANS))])
                    .font_weight(FontWeightKeyword::Thin)
                    .font_size(30.0);
                Element::new(cx).width(Stretch(1.0));
            })
            .height(Pixels(50.0))
            .width(Percentage(100.0));

            ScrollView::new(cx, |cx| {
                // This looks better if it's flush at the top, and then we'll just add some padding
                // at the top of the scroll view
                GenericUi::new(cx, Data::params);
            })
            .width(Percentage(100.0))
            .top(Pixels(5.0));
        })
        .width(Percentage(100.0));

        nih_plug_vizia::widgets::ResizeHandle::new(cx);
    })
}
