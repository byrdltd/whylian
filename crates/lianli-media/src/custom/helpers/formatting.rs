use lianli_shared::template::WidgetKind;

pub fn format_sensor_readout(kind: &WidgetKind, raw: f32) -> (String, i32) {
    match kind {
        WidgetKind::ValueText { format, unit, .. } => {
            let text = render_value_format(format, raw);
            let quantized = (raw * 10.0).round() as i32;
            (format!("{text}{unit}"), quantized)
        }
        WidgetKind::RadialGauge {
            value_min,
            value_max,
            ..
        }
        | WidgetKind::VerticalBar {
            value_min,
            value_max,
            ..
        }
        | WidgetKind::HorizontalBar {
            value_min,
            value_max,
            ..
        }
        | WidgetKind::Speedometer {
            value_min,
            value_max,
            ..
        }
        | WidgetKind::Sparkline {
            value_min,
            value_max,
            ..
        } => {
            let span = (value_max - value_min).abs().max(f32::EPSILON);
            let q = (((raw - value_min) / span) * 1000.0).round() as i32;
            (String::new(), q)
        }
        _ => (String::new(), 0),
    }
}

pub fn render_value_format(fmt: &str, value: f32) -> String {
    if let Some(open) = fmt.find('{') {
        if let Some(close_rel) = fmt[open..].find('}') {
            let close = open + close_rel;
            let spec = &fmt[open + 1..close];
            let decimals = spec
                .strip_prefix(":.")
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(0);
            let prefix = &fmt[..open];
            let suffix = &fmt[close + 1..];
            return format!("{prefix}{:.*}{suffix}", decimals, value);
        }
    }
    format!("{:.0}", value)
}
