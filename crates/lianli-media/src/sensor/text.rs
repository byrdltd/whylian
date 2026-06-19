use super::bitmap_glyphs::glyph_pattern;
use ab_glyph::{point, Font, FontVec, PxScale, ScaleFont};
use image::{Rgb, RgbImage};

pub(super) struct TextRenderParams<'a> {
    pub label: &'a str,
    pub unit: &'a str,
    pub color: [u8; 3],
    pub value_size: f32,
    pub unit_size: f32,
    pub label_size: f32,
    pub value_offset: i32,
    pub unit_offset: i32,
    pub label_offset: i32,
    pub value_text: &'a str,
}

pub(super) fn draw_sensor_text_ttf(
    image: &mut RgbImage,
    width: u32,
    height: u32,
    params: TextRenderParams,
    font: &FontVec,
) {
    draw_text_centered(
        image,
        width,
        height,
        params.value_text,
        params.value_size,
        params.color,
        params.value_offset,
        font,
    );
    draw_text_centered(
        image,
        width,
        height,
        params.unit,
        params.unit_size,
        params.color,
        params.unit_offset,
        font,
    );
    draw_text_centered(
        image,
        width,
        height,
        params.label,
        params.label_size,
        params.color,
        params.label_offset,
        font,
    );
}

fn draw_text_centered(
    image: &mut RgbImage,
    width: u32,
    height: u32,
    text: &str,
    size: f32,
    color: [u8; 3],
    offset_y: i32,
    font: &FontVec,
) {
    if size <= 0.0 || text.is_empty() {
        return;
    }

    let scale = PxScale::from(size);
    let scaled = font.as_scaled(scale);

    let mut cursor_x = 0.0_f32;
    let mut positioned: Vec<ab_glyph::Glyph> = Vec::with_capacity(text.len());
    for ch in text.chars() {
        let glyph_id = scaled.glyph_id(ch);
        let glyph = glyph_id.with_scale_and_position(scale, point(cursor_x, scaled.ascent()));
        positioned.push(glyph);
        cursor_x += scaled.h_advance(glyph_id);
    }

    let text_width = cursor_x;
    let start_x = ((width as f32 - text_width) / 2.0) as i32;
    let start_y = (height as i32 / 2) + offset_y;

    for glyph in positioned {
        if let Some(outlined) = scaled.outline_glyph(glyph) {
            let bb = outlined.px_bounds();
            outlined.draw(|gx, gy, gv| {
                let x = start_x + bb.min.x as i32 + gx as i32;
                let y = start_y + bb.min.y as i32 + gy as i32;
                if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                    let px = image.get_pixel_mut(x as u32, y as u32);
                    let alpha = gv;
                    px.0[0] = ((color[0] as f32 * alpha) + (px.0[0] as f32 * (1.0 - alpha))) as u8;
                    px.0[1] = ((color[1] as f32 * alpha) + (px.0[1] as f32 * (1.0 - alpha))) as u8;
                    px.0[2] = ((color[2] as f32 * alpha) + (px.0[2] as f32 * (1.0 - alpha))) as u8;
                }
            });
        }
    }
}

pub(super) fn draw_sensor_text_fallback(
    image: &mut RgbImage,
    width: u32,
    height: u32,
    params: TextRenderParams,
) {
    let value_scale = (params.value_size / 4.0).max(4.0) as u32;
    let unit_scale = (params.unit_size / 4.0).max(3.0) as u32;
    let label_scale = (params.label_size / 4.0).max(3.0) as u32;

    draw_text_center_bitmap(
        image,
        width,
        height,
        params.value_text,
        value_scale,
        params.color,
        params.value_offset,
    );
    draw_text_center_bitmap(
        image,
        width,
        height,
        params.unit,
        unit_scale,
        params.color,
        params.unit_offset,
    );
    draw_text_center_bitmap(
        image,
        width,
        height,
        params.label,
        label_scale,
        params.color,
        params.label_offset,
    );
}

fn draw_text_center_bitmap(
    image: &mut RgbImage,
    width: u32,
    height: u32,
    text: &str,
    scale: u32,
    color: [u8; 3],
    offset_y: i32,
) {
    if scale == 0 {
        return;
    }
    let glyphs: Vec<[u8; 7]> = text.chars().map(glyph_pattern).collect();
    if glyphs.is_empty() {
        return;
    }
    let glyph_width = 5 * scale;
    let spacing = scale;
    let total_width = (glyphs.len() as u32 * (glyph_width + spacing) - spacing).min(width);
    let start_x = ((width - total_width) / 2) as i32;
    let start_y = ((height as i32) / 2) + offset_y - ((7 * scale) as i32 / 2);

    for (i, bitmap) in glyphs.iter().enumerate() {
        let base_x = start_x + i as i32 * (glyph_width as i32 + spacing as i32);
        draw_bitmap_character(image, width, height, base_x, start_y, *bitmap, scale, color);
    }
}

fn draw_bitmap_character(
    image: &mut RgbImage,
    width: u32,
    height: u32,
    base_x: i32,
    base_y: i32,
    bitmap: [u8; 7],
    scale: u32,
    color: [u8; 3],
) {
    for (row, mask) in bitmap.iter().enumerate() {
        for col in 0..5 {
            if (mask >> (4 - col)) & 1 == 1 {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let x = base_x + (col * scale) as i32 + dx as i32;
                        let y = base_y + (row as i32 * scale as i32) + dy as i32;
                        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                            image.put_pixel(x as u32, y as u32, Rgb(color));
                        }
                    }
                }
            }
        }
    }
}
