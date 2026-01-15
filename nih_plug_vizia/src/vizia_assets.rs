//! Registration functions for Vizia's built-in fonts. These are not enabled by default in
//! `nih_plug_vizia` to save on binary size.

use vizia::prelude::*;

/// The font name for the Roboto font family. Comes in regular, bold, and italic variations.
/// Register the variations you want to use with [`register_roboto()`], [`register_roboto_bold()`],
/// and [`register_roboto_italic()`] first. Use the font weight and font style properties to select
/// a specific variation.
///
/// Note: Vizia 3 no longer includes Roboto fonts by default. You can use other fonts or provide
/// your own font files.
pub const ROBOTO: &str = "Roboto";
/// The font name for the icon font (tabler-icons), needs to be registered using
/// [`register_tabler_icons()`] first.
///
/// Note: Vizia 3 no longer includes tabler-icons by default. You can use other icon fonts or
/// provide your own icon font file.
pub const TABLER_ICONS: &str = "tabler-icons";

pub fn register_roboto(_cx: &mut Context) {
    // TODO: In Vizia 3, fonts are no longer built-in. Users should provide their own fonts.
    // cx.add_font_mem(fonts::ROBOTO_REGULAR);
}
pub fn register_roboto_bold(_cx: &mut Context) {
    // TODO: In Vizia 3, fonts are no longer built-in. Users should provide their own fonts.
    // cx.add_font_mem(fonts::ROBOTO_BOLD);
}
pub fn register_roboto_italic(_cx: &mut Context) {
    // TODO: In Vizia 3, fonts are no longer built-in. Users should provide their own fonts.
    // cx.add_font_mem(fonts::ROBOTO_ITALIC);
}
pub fn register_tabler_icons(_cx: &mut Context) {
    // TODO: In Vizia 3, fonts are no longer built-in. Users should provide their own fonts.
    // cx.add_font_mem(fonts::TABLER_ICONS);
}
