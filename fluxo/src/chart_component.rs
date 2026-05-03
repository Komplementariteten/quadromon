use std::sync::Arc;
use vello::kurbo::{Affine, Rect, RoundedRect, Stroke};
use vello::peniko::{Blob, Brush, Color, Fill, FontData};
use vello::{Glyph, Scene};
use crate::components::Component;
use crate::{DEFAULT_MARGIN, DEFAULT_RECT_RADIUS, DEFAULT_STROKE_WIDTH};
use crate::axis::render_axis;
use crate::text_render::{render_text_normal, render_text_small, render_text_small_color, Alignment};

const CHART_HEIGHT: u32 = 100;

const PLOT_HEIGHT: u32 = 60;
const BAR_WIDTH: usize = 4;

const NUMBER_OFF_BARS: u32 = 20;

static COLOR_BLUE: Color = Color::new([0.2078, 0.5647, 0.8461, 0.8]);
static COLOR_RED: Color = Color::new([0.75, 0.15, 0.15, 0.7]);

pub fn render_chart(scene: &mut Scene, title: &str, values: Vec<f64>, offset: u32, width: u32) -> u32 {
    let axis_pos = render_axis(scene, None, offset as f64, width, CHART_HEIGHT);
    let max_value = values.iter().max_by(|a, b| a.total_cmp(b)).unwrap().clone();
    add_plot(scene, values.clone(), max_value, offset, width, axis_pos, CHART_HEIGHT);
    add_title(scene, title, offset as f64 + (2. * DEFAULT_MARGIN) + DEFAULT_STROKE_WIDTH);
    add_border(scene, offset as f64, width as f64, CHART_HEIGHT as f64);
    let max_value_s = format!("{:.4}", max_value);
    let max_v_offset_y = offset as f64 + (4. * DEFAULT_MARGIN);
    let max_v_offset_x = (width as f64) - (4. * DEFAULT_MARGIN);
    render_text_small_color(scene, max_value_s.as_str(), Alignment::Right, max_v_offset_x, max_v_offset_y, Color::WHITE);
    return CHART_HEIGHT;
}


fn add_title(scene: &mut Scene, title: &str, offset: f64){
    render_text_normal(scene, title, (2. * DEFAULT_MARGIN) + DEFAULT_STROKE_WIDTH, offset);
}

fn add_border(scene: &mut Scene, offset: f64, width: f64, height: f64) {
    let border_color = Color::new([0.25, 0.25, 0.25, 1.0]);
    let stroke = Stroke::new(DEFAULT_STROKE_WIDTH);
    let rect = RoundedRect::new(
        DEFAULT_MARGIN,
        offset + DEFAULT_MARGIN,
        width - (2.0 * DEFAULT_MARGIN),
        offset + height - (2.0 * DEFAULT_MARGIN),
        DEFAULT_RECT_RADIUS,
    );
    scene.stroke(&stroke, Affine::IDENTITY, border_color, None, &rect)
}

fn add_plot(scene: &mut Scene, values: Vec<f64>, max_value: f64, offset: u32, width: u32, height: u32, axis_pos: u32) {
    let mut  bar_values = vec![];
    let bar_y_pos = axis_pos - (4. * DEFAULT_MARGIN) as u32;
    let top_margin = offset as f64 + (6. * DEFAULT_MARGIN);
    let left_margin = 3. * DEFAULT_MARGIN;
    if values.len() < NUMBER_OFF_BARS as usize {
        for value in values {
            bar_values.push(value_to_x_offset(top_margin, value, height, max_value));
        }
    } else {
        let parts = values.len() / (NUMBER_OFF_BARS as usize);
        let mut count = 0;
        let mut sum: f64 = 0.0;
        for i in 0..values.len() {
            if count >= parts {
                let avg = sum / (parts as f64);
                bar_values.push(value_to_x_offset(top_margin, avg, height, max_value));
                sum = 0.;
                count = 0;
            }
            sum += values[i];
            count += 1;
        }

        if count > 0 {
            let avg = sum / (count as f64);
            bar_values.push(value_to_x_offset(top_margin, avg, height, max_value));
        }
    }

    let rect_color = COLOR_BLUE;

    for i in 0..bar_values.len() {
        let rect = Rect::new(left_margin + (i * BAR_WIDTH) as f64, bar_values[i as usize],  left_margin + ((i + 1) * BAR_WIDTH) as f64, bar_y_pos as f64 );
        scene.fill(Fill::NonZero, Affine::IDENTITY, rect_color, None, &rect);
    }

}

fn value_to_x_offset(total_offset: f64, value: f64, height: u32, max_value: f64) -> f64 {
    let ratio = 1. - (value / max_value);
    total_offset + (height as f64 * ratio)
}