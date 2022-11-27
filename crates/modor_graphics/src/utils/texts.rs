use crate::Alignment;
use ab_glyph::{Font, FontVec, Glyph, PxScaleFont, ScaleFont};
use image::{DynamicImage, Rgba, RgbaImage};

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn generate_texture(
    string: &str,
    alignment: Alignment,
    font_height: f32,
    font: &FontVec,
) -> RgbaImage {
    let font = font.as_scaled(font_height);
    let line_widths = calculate_line_widths(font, string);
    let width = line_widths.iter().fold(0.0_f32, |a, &b| a.max(b)).max(1.);
    let height = calculate_image_height(font, string).max(1);
    let mut image = DynamicImage::new_rgba8(width.ceil() as u32, height).to_rgba8();
    render_glyphs(font, string, width, &line_widths, alignment, &mut image);
    image
}

fn calculate_line_widths(font: PxScaleFont<&FontVec>, string: &str) -> Vec<f32> {
    string
        .lines()
        .map(|l| {
            let mut last_glyph: Option<Glyph> = None;
            l.chars()
                .filter(|c| !c.is_control())
                .map(|c| {
                    let glyph = font.scaled_glyph(c);
                    let width = font.h_advance(glyph.id)
                        + last_glyph
                            .as_ref()
                            .map_or(0., |g| font.kern(g.id, glyph.id));
                    last_glyph = Some(glyph);
                    width
                })
                .sum::<f32>()
        })
        .collect()
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn calculate_image_height(font: PxScaleFont<&FontVec>, string: &str) -> u32 {
    let line_count = string.lines().count() + usize::from(string.ends_with('\n'));
    let gap_count = line_count.saturating_sub(1);
    let height = font.height().mul_add(
        line_count as f32,
        font.line_gap().mul_add(gap_count as f32, 1.),
    );
    1.max(height.ceil() as u32)
}

fn render_glyphs(
    font: PxScaleFont<&FontVec>,
    string: &str,
    width: f32,
    line_widths: &[f32],
    alignment: Alignment,
    image: &mut RgbaImage,
) {
    let v_advance = font.height() + font.line_gap();
    let mut cursor_y = font.ascent();
    for (line, &line_width) in string.lines().zip(line_widths) {
        let mut cursor_x = match alignment {
            Alignment::Left | Alignment::TopLeft | Alignment::BottomLeft => 0.,
            Alignment::Center | Alignment::Top | Alignment::Bottom => (width - line_width) / 2.,
            Alignment::Right | Alignment::TopRight | Alignment::BottomRight => width - line_width,
        };
        let mut last_glyph_id = None;
        for character in line.chars() {
            if character.is_control() {
                continue;
            }
            let mut glyph = font.scaled_glyph(character);
            glyph.position = ab_glyph::point(cursor_x, cursor_y);
            cursor_x += font.h_advance(glyph.id);
            if let Some(last_glyph_id) = last_glyph_id {
                cursor_x += font.kern(last_glyph_id, glyph.id);
            }
            last_glyph_id = Some(glyph.id);
            render_glyph(font, glyph, image);
        }
        cursor_y += v_advance;
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn render_glyph(font: PxScaleFont<&FontVec>, glyph: Glyph, image: &mut RgbaImage) {
    if let Some(outlined) = font.outline_glyph(glyph) {
        let bounds = outlined.px_bounds();
        outlined.draw(|x, y, v| {
            let x = x + bounds.min.x as u32;
            let y = y + bounds.min.y as u32;
            if let Some(px) = image.get_pixel_mut_checked(x, y) {
                *px = Rgba([255, 255, 255, px.0[3].saturating_add((v * 255.) as u8)]);
            }
        });
    }
}
