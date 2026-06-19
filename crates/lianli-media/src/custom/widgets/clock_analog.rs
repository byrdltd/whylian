//! Analog clock with configurable face, ticks, numbers, and three hands.

use super::super::helpers::{draw_text_widget, fast_overlay};
use ab_glyph::FontVec;
use chrono::{Local, Timelike};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{
    draw_antialiased_line_segment_mut, draw_filled_circle_mut, draw_polygon_mut,
};
use imageproc::pixelops::interpolate;
use imageproc::point::Point;
use lianli_shared::template::TextAlign;
use std::f32::consts::PI;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn draw(
    sub: &mut RgbaImage,
    face_color: [u8; 4],
    tick_color: [u8; 4],
    minor_tick_color: [u8; 4],
    hour_hand_color: [u8; 4],
    minute_hand_color: [u8; 4],
    second_hand_color: [u8; 4],
    hub_color: [u8; 4],
    numbers_color: [u8; 4],
    numbers_font: &FontVec,
    numbers_font_size: f32,
    show_seconds: bool,
    show_hour_ticks: bool,
    show_minor_ticks: bool,
    show_numbers: bool,
    hour_hand_width: f32,
    minute_hand_width: f32,
    second_hand_width: f32,
    hour_hand_length_pct: f32,
    minute_hand_length_pct: f32,
    second_hand_length_pct: f32,
    hour_tick_length_pct: f32,
    minor_tick_length_pct: f32,
    hour_tick_width: f32,
    minor_tick_width: f32,
    hub_radius: f32,
    uniform_scale: f32,
) {
    let (w, h) = (sub.width() as f32, sub.height() as f32);
    let center = (w / 2.0, h / 2.0);
    let r_outer = (w.min(h) / 2.0).max(1.0);

    if face_color[3] > 0 {
        draw_filled_circle_mut(
            sub,
            (center.0 as i32, center.1 as i32),
            r_outer as i32,
            Rgba(face_color),
        );
    }

    if show_minor_ticks && minor_tick_color[3] > 0 {
        let tick_rgba = Rgba(minor_tick_color);
        let inner_r = r_outer * (1.0 - minor_tick_length_pct.clamp(0.0, 0.5));
        let width = (minor_tick_width * uniform_scale).max(1.0) as i32;
        for i in 0..60 {
            if i % 5 == 0 {
                continue;
            }
            let angle = minute_to_angle_rad(i as f32);
            draw_radial_line(sub, center, inner_r, r_outer, angle, width, tick_rgba);
        }
    }

    if show_hour_ticks && tick_color[3] > 0 {
        let tick_rgba = Rgba(tick_color);
        let inner_r = r_outer * (1.0 - hour_tick_length_pct.clamp(0.0, 0.5));
        let width = (hour_tick_width * uniform_scale).max(1.0) as i32;
        for i in 0..12 {
            let angle = hour_mark_to_angle_rad(i as f32);
            draw_radial_line(sub, center, inner_r, r_outer, angle, width, tick_rgba);
        }
    }

    if show_numbers && numbers_color[3] > 0 {
        let num_radius = r_outer * (1.0 - hour_tick_length_pct.clamp(0.0, 0.5) - 0.08).max(0.1);
        let box_w = (numbers_font_size * 2.0).max(16.0) as u32;
        let box_h = (numbers_font_size * 1.4).max(16.0) as u32;
        for i in 1..=12 {
            let angle = hour_mark_to_angle_rad(i as f32);
            let nx = center.0 + num_radius * angle.cos();
            let ny = center.1 + num_radius * angle.sin();
            let tl_x = (nx - box_w as f32 / 2.0).round() as i32;
            let tl_y = (ny - box_h as f32 / 2.0).round() as i32;
            let mut glyph_canvas = RgbaImage::from_pixel(box_w, box_h, Rgba([0, 0, 0, 0]));
            draw_text_widget(
                &mut glyph_canvas,
                &format!("{i}"),
                numbers_font,
                numbers_font_size * uniform_scale,
                numbers_color,
                TextAlign::Center,
                box_w,
                box_h,
                0.0,
            );
            fast_overlay(sub, &glyph_canvas, tl_x as i64, tl_y as i64);
        }
    }

    let now = Local::now();
    let hour = now.hour() % 12;
    let minute = now.minute();
    let second = now.second();
    let nano = now.nanosecond();
    let sec_f = second as f32 + nano as f32 / 1_000_000_000.0;
    let min_f = minute as f32 + sec_f / 60.0;
    let hour_f = hour as f32 + min_f / 60.0;

    let hour_angle = hour_mark_to_angle_rad(hour_f);
    let minute_angle = minute_to_angle_rad(min_f);
    let second_angle = minute_to_angle_rad(sec_f);

    draw_hand(
        sub,
        center,
        hour_angle,
        r_outer * hour_hand_length_pct.clamp(0.1, 1.2),
        (hour_hand_width * uniform_scale).max(2.0),
        hour_hand_color,
    );
    draw_hand(
        sub,
        center,
        minute_angle,
        r_outer * minute_hand_length_pct.clamp(0.1, 1.2),
        (minute_hand_width * uniform_scale).max(2.0),
        minute_hand_color,
    );
    if show_seconds {
        draw_hand(
            sub,
            center,
            second_angle,
            r_outer * second_hand_length_pct.clamp(0.1, 1.2),
            (second_hand_width * uniform_scale).max(1.0),
            second_hand_color,
        );
    }

    if hub_color[3] > 0 {
        let r = (hub_radius * uniform_scale).round().max(1.0) as i32;
        draw_filled_circle_mut(sub, (center.0 as i32, center.1 as i32), r, Rgba(hub_color));
    }
}

fn hour_mark_to_angle_rad(hour_mark: f32) -> f32 {
    (hour_mark / 12.0) * 2.0 * PI - PI / 2.0
}

fn minute_to_angle_rad(minute: f32) -> f32 {
    (minute / 60.0) * 2.0 * PI - PI / 2.0
}

fn draw_radial_line(
    img: &mut RgbaImage,
    center: (f32, f32),
    r_in: f32,
    r_out: f32,
    angle_rad: f32,
    width: i32,
    color: Rgba<u8>,
) {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    let p_in = (center.0 + r_in * cos_a, center.1 + r_in * sin_a);
    let p_out = (center.0 + r_out * cos_a, center.1 + r_out * sin_a);
    let half = width as f32 / 2.0;
    let orth = angle_rad + PI / 2.0;
    let ox = orth.cos() * half;
    let oy = orth.sin() * half;
    if width <= 1 {
        draw_antialiased_line_segment_mut(
            img,
            (p_in.0 as i32, p_in.1 as i32),
            (p_out.0 as i32, p_out.1 as i32),
            color,
            interpolate,
        );
        return;
    }
    let poly = vec![
        Point::new((p_in.0 - ox) as i32, (p_in.1 - oy) as i32),
        Point::new((p_out.0 - ox) as i32, (p_out.1 - oy) as i32),
        Point::new((p_out.0 + ox) as i32, (p_out.1 + oy) as i32),
        Point::new((p_in.0 + ox) as i32, (p_in.1 + oy) as i32),
    ];
    draw_polygon_mut(img, &poly, color);
}

fn draw_hand(
    img: &mut RgbaImage,
    center: (f32, f32),
    angle_rad: f32,
    length: f32,
    width: f32,
    color: [u8; 4],
) {
    if color[3] == 0 {
        return;
    }
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    let tip = (center.0 + length * cos_a, center.1 + length * sin_a);
    let orth = angle_rad + PI / 2.0;
    let half = width / 2.0;
    let ox = orth.cos() * half;
    let oy = orth.sin() * half;
    let poly = vec![
        Point::new((center.0 - ox) as i32, (center.1 - oy) as i32),
        Point::new((tip.0 - ox) as i32, (tip.1 - oy) as i32),
        Point::new((tip.0 + ox) as i32, (tip.1 + oy) as i32),
        Point::new((center.0 + ox) as i32, (center.1 + oy) as i32),
    ];
    draw_polygon_mut(img, &poly, Rgba(color));
}
