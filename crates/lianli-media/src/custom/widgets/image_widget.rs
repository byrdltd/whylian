//! Static image overlay with optional opacity.

use super::super::helpers::blit_with_opacity;
use super::WidgetState;
use image::RgbaImage;

pub(in super::super) fn draw(sub: &mut RgbaImage, state: &WidgetState, opacity: f32) {
    if let Some(img) = &state.loaded_image {
        blit_with_opacity(sub, img, opacity);
    }
}
