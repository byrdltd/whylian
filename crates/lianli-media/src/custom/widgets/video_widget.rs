//! Looping video frame overlay.

use super::super::helpers::blit_with_opacity;
use super::WidgetState;
use image::RgbaImage;

pub(in super::super) fn draw(sub: &mut RgbaImage, state: &WidgetState, opacity: f32) {
    if let Some(frames) = &state.video_frames {
        if !frames.is_empty() {
            let idx = state
                .last_video_frame_idx
                .unwrap_or(0)
                .min(frames.len() - 1);
            blit_with_opacity(sub, &frames[idx], opacity);
        }
    }
}
