use std::sync::Arc;
use skrifa::MetadataProvider;
use skrifa::raw::FileRef;
use vello::peniko::{Blob, Brush, Color, Fill, FontData, StyleRef};
use vello::{Glyph, Scene};
use vello::kurbo::Affine;
use vello::peniko::color::{AlphaColor, Srgb};
use vello::wgpu::naga::Scalar;

const ROBOTO_FONT: &[u8] = include_bytes!("../assets/fonts/SpaceMono-Regular.ttf");

const NORMAL_FONT_SIZE: f32 = 14.0;
const BIG_FONT_SIZE: f32 = 24.0;
const SMALL_FONT_SIZE: f32 = 10.0;
pub fn render_text_default(scene: &mut Scene, size: f32, text: &str, x_off: f32, y_off: f32) -> f32 {
    let title_color = Color::new([0.85, 0.85, 0.85, 1.0]);
    let font_data = FontData::new(Blob::new(Arc::new(ROBOTO_FONT)), 0);
    let brush = Brush::Solid(title_color);
    render_text_complex(
        scene,
        size,
        text,
        x_off,
        y_off,
        &brush,
        &font_data,
        Fill::NonZero,
        None,
        None,
        None
    )
}

pub fn render_text_vertical(scene: &mut Scene, text: &str, x_off: f32, y_off: f32) -> f32 {
    let title_color = Color::new([0.85, 0.85, 0.85, 1.0]);
    let font_data = FontData::new(Blob::new(Arc::new(ROBOTO_FONT)), 0);
    let brush = Brush::Solid(title_color);
    render_text_complex(
        scene,
        NORMAL_FONT_SIZE,
        text,
        x_off,
        y_off,
        &brush,
        &font_data,
        Fill::NonZero,
        Some(Affine::translate((110.0, 800.0))),
        None,
        None
    )
}

pub fn render_text_normal(scene: &mut Scene, text: &str, x_off: f64, y_off: f64) -> f64 {
    render_text_default(scene, NORMAL_FONT_SIZE, text, x_off as f32, y_off as f32) as f64
}

pub fn render_text_small(scene: &mut Scene, text: &str, x_off: f64, y_off: f64) -> f64 {
    render_text_default(scene, SMALL_FONT_SIZE, text, x_off as f32, y_off as f32) as f64
}

fn render_text_complex<'a>(scene: &'a mut Scene, size: f32, text: &str, x_off: f32, y_off: f32,
                           brush: &'a Brush, font: &FontData,
                           style: impl Into<StyleRef<'a>>, transform: Option<Affine>,
                           variations: Option<&[(&str, f32)]>, glyph_transform: Option<Affine>) -> f32 {
    let font_opt  = match FileRef::new(font.data.as_ref()).ok().expect("Failed to load Font as ref.") {
        FileRef::Font(f) => Some(f),
        FileRef::Collection(c) => c.get(font.index).ok()
    };
    let font_ref = font_opt.expect("Failed to load Font Ref");
    let charmap = font_ref.charmap();
    let axes = font_ref.axes();
    let mut variation: &[(&str, f32)] = &[];
    if let Some(provided_var) = variations {
        variation = provided_var
    }
    let var_loc = axes.location(variation.iter().copied());
    let font_size = skrifa::instance::Size::new(size);
    let metric = font_ref.metrics(font_size, &var_loc);
    let line_height = metric.ascent - metric.descent + metric.leading;
    let glyph_metric = font_ref.glyph_metrics(font_size, &var_loc);
    let mut pen_x = x_off;
    let mut pen_y = y_off + line_height;
    let mut glyph_draw = scene.draw_glyphs(font).font_size(size).brush(brush).glyph_transform(glyph_transform);
    if let Some(provided_transform) = transform {
        glyph_draw = glyph_draw.transform(provided_transform);
    }
    glyph_draw.draw(style, text.chars().filter_map(| c| {
        if c == '\n' {
            pen_y += line_height;
            pen_x = x_off;
            return None;
        }

        let gid = charmap.map(c).unwrap_or_default();
        let av = glyph_metric.advance_width(gid).unwrap_or_default();
        let x = pen_x;
        pen_x += av;
        Some(Glyph {
            id: gid.to_u32(),
            x,
            y: pen_y,
        })
    }));

    pen_y
}