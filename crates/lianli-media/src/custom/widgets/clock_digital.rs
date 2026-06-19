//! Digital clock — renders `chrono::Local::now()` via a strftime format string.

use super::super::helpers::draw_text_widget;
use ab_glyph::FontVec;
use chrono::Local;
use image::RgbaImage;
use lianli_shared::template::TextAlign;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn draw(
    sub: &mut RgbaImage,
    format: &str,
    font: &FontVec,
    size: f32,
    color: [u8; 4],
    align: TextAlign,
    ww: u32,
    wh: u32,
    letter_spacing: f32,
) {
    let now = Local::now();
    let text = now.format(format).to_string();
    draw_text_widget(sub, &text, font, size, color, align, ww, wh, letter_spacing);
}
