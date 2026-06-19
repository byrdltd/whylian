//! Shared helpers for Custom widget rendering.

mod drawing;
mod fonts;
mod formatting;
mod text;

pub(super) use drawing::{
    blit_with_opacity, draw_annulus, fast_overlay, fast_resize_rgba, fill_rect_clipped_rounded,
    fill_rounded_rect, fit_image, range_color, range_color_blended, unit_interval,
};
pub(super) use fonts::{load_font_from_disk, resolve_font, widget_font_refs};
pub(super) use formatting::{format_sensor_readout, render_value_format};
pub(super) use text::draw_text_widget;

use lianli_shared::media::SensorSourceConfig;
use lianli_shared::sensors::{resolve_sensor, ResolvedSensor, SensorInfo};
use lianli_shared::template::{Widget, WidgetKind};

pub(super) fn widget_sensor_source(kind: &WidgetKind) -> Option<&SensorSourceConfig> {
    match kind {
        WidgetKind::ValueText { source, .. }
        | WidgetKind::RadialGauge { source, .. }
        | WidgetKind::VerticalBar { source, .. }
        | WidgetKind::HorizontalBar { source, .. }
        | WidgetKind::Speedometer { source, .. }
        | WidgetKind::Sparkline { source, .. } => Some(source),
        _ => None,
    }
}

pub(super) fn resolve_sensor_source(
    source: &SensorSourceConfig,
    all_sensors: &[SensorInfo],
) -> Option<ResolvedSensor> {
    if let SensorSourceConfig::Constant { value } = source {
        return Some(ResolvedSensor::Constant(*value));
    }
    let target = source.to_sensor_source();
    let divider = all_sensors
        .iter()
        .find(|s| s.source == target)
        .map(|s| s.divider)
        .unwrap_or(1);
    resolve_sensor(&target, divider)
}

pub(super) fn widget_size_px(widget: &Widget, uniform_scale: f32) -> (u32, u32) {
    (
        (widget.width * uniform_scale).round().max(1.0) as u32,
        (widget.height * uniform_scale).round().max(1.0) as u32,
    )
}
