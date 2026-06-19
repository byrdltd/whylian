use crate::common::MediaError;
use ab_glyph::FontVec;
use lianli_shared::template::{FontRef, WidgetKind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn load_font_from_disk(path: &Path) -> Result<FontVec, MediaError> {
    let bytes = std::fs::read(path)
        .map_err(|e| MediaError::Sensor(format!("font '{}' read failed: {e}", path.display())))?;
    FontVec::try_from_vec(bytes)
        .map_err(|e| MediaError::Sensor(format!("font '{}' parse failed: {e}", path.display())))
}

pub fn widget_font_refs(kind: &WidgetKind) -> Vec<&FontRef> {
    match kind {
        WidgetKind::Label { font, .. } | WidgetKind::ValueText { font, .. } => vec![font],
        WidgetKind::ClockDigital { font, .. } => vec![font],
        WidgetKind::ClockAnalog { numbers_font, .. } => vec![numbers_font],
        WidgetKind::Sparkline {
            axis_label_font, ..
        } => vec![axis_label_font],
        _ => Vec::new(),
    }
}

pub fn resolve_font<'a>(
    font_ref: &FontRef,
    fonts: &'a HashMap<PathBuf, FontVec>,
    default: &'a FontVec,
) -> &'a FontVec {
    if let Some(p) = &font_ref.path {
        if let Some(f) = fonts.get(p) {
            return f;
        }
    }
    default
}
