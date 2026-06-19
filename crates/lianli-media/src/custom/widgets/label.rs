//! Static text label.

use super::super::helpers::draw_text_widget;
use ab_glyph::FontVec;
use image::RgbaImage;
use lianli_shared::template::TextAlign;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn draw(
    sub: &mut RgbaImage,
    text: &str,
    font: &FontVec,
    size: f32,
    color: [u8; 4],
    align: TextAlign,
    ww: u32,
    wh: u32,
    letter_spacing: f32,
) {
    draw_text_widget(sub, text, font, size, color, align, ww, wh, letter_spacing);
}
