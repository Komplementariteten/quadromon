use crate::text_render::render_text_small;
use crate::{DEFAULT_MARGIN, DEFAULT_STROKE_WIDTH};
use vello::Scene;
use vello::kurbo::{Affine, Line, Point, Stroke};
use vello::peniko::Color;
use vello::peniko::color::{AlphaColor, Srgb};

pub const DEFAULT_AXIS_WIDTH: f64 = 1.25;

#[derive(Debug, Clone)]
pub struct AxisOption {
    show_x: bool,
    show_y: bool,
    x_ticks: Option<usize>,
    y_ticks: Option<usize>,
    x_axis_title: Option<String>,
    y_axis_title: Option<String>,
    color: AlphaColor<Srgb>,
}

impl Default for AxisOption {
    fn default() -> Self {
        AxisOption {
            show_y: false,
            show_x: true,
            x_ticks: None,
            y_ticks: None,
            y_axis_title: None,
            x_axis_title: None,
            color: Color::new([1.0, 1.0, 0.5, 0.75]),
        }
    }
}

pub fn render_axis(
    scene: &mut Scene,
    opt: Option<&AxisOption>,
    offset: f64,
    width: u32,
    height: u32,
) -> u32 {
    let mut used_opt: AxisOption = AxisOption::default();
    if let Some(x) = opt {
        used_opt = x.clone();
    }

    let mut y_offset = offset + height as f64 - DEFAULT_MARGIN;
    if let Some(title) = used_opt.x_axis_title {
        let height = render_text_small(scene, title.as_str(), 5. * DEFAULT_MARGIN, y_offset);
        y_offset -= (height + (2. * DEFAULT_MARGIN));
    }
    let start: Point = ((3. * DEFAULT_MARGIN), y_offset - (2. * DEFAULT_MARGIN)).into();
    let end: Point = (
        width as f64 - (8. * DEFAULT_MARGIN),
        y_offset - (2. * DEFAULT_MARGIN),
    )
        .into();
    let line = Line::new(start, end);
    let stroke = Stroke::new(DEFAULT_AXIS_WIDTH);
    scene.stroke(&stroke, Affine::IDENTITY, used_opt.color, None, &line);
    y_offset as u32
}
