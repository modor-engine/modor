use crate::components::font::{FontKey, FontRegistry};
use crate::Font;
use ab_glyph::{Font as AbFont, FontVec, Glyph, PxScaleFont, ScaleFont};
use modor::{Query, SingleMut};
use modor_graphics_new2::{Size, Texture, TextureSource};
use modor_resources::{IntoResourceKey, ResourceKey};

const TEXTURE_PADDING_PX: u32 = 1;

#[derive(Component, Debug)]
pub struct Text {
    pub content: String,
    pub font_height: f32,
    pub font_key: ResourceKey,
    pub alignment: Alignment,
    old_content: String,
    old_font_height: f32,
    old_font_key: ResourceKey,
    old_alignment: Alignment,
}

#[systems]
impl Text {
    pub fn new(content: impl Into<String>, font_height: f32) -> Self {
        Self {
            content: content.into(),
            font_height,
            font_key: FontKey::Default.into_key(),
            alignment: Alignment::default(),
            old_content: String::new(),
            old_font_height: font_height,
            old_font_key: FontKey::Default.into_key(),
            old_alignment: Alignment::default(),
        }
    }

    pub fn with_font(mut self, font: impl IntoResourceKey) -> Self {
        self.font_key = font.into_key();
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    #[run_after(component(FontRegistry), component(Font))]
    fn update(
        &mut self,
        texture: &mut Texture,
        (mut font_registry, fonts): (SingleMut<'_, FontRegistry>, Query<'_, &Font>),
    ) {
        if let Some(font) = font_registry.get(&self.font_key, &fonts) {
            if self.has_changed() || font.is_just_loaded {
                let font = font.get().as_scaled(self.font_height);
                let line_widths = self.line_widths(font);
                let width = line_widths.iter().fold(0.0_f32, |a, &b| a.max(b)).max(1.);
                let height = self.height(font).max(1);
                let size = Size::new(
                    width.ceil() as u32 + TEXTURE_PADDING_PX * 2,
                    height + TEXTURE_PADDING_PX * 2,
                );
                let mut buffer = vec![0; (size.width * size.height) as usize * 4];
                self.render_glyphs(font, width, &line_widths, &mut buffer, size);
                texture.set_source(TextureSource::Buffer(size, buffer));
                self.set_as_unchanged();
            }
        }
    }

    fn has_changed(&mut self) -> bool {
        (
            &self.content,
            self.font_height,
            &self.font_key,
            self.alignment,
        ) != (
            &self.old_content,
            self.old_font_height,
            &self.old_font_key,
            self.old_alignment,
        )
    }

    fn set_as_unchanged(&mut self) {
        self.old_content = self.content.clone();
        self.old_font_height = self.font_height;
        self.old_font_key = self.font_key.clone();
        self.old_alignment = self.alignment;
    }

    fn line_widths(&self, font: PxScaleFont<&FontVec>) -> Vec<f32> {
        self.content
            .lines()
            .map(|l| Self::line_width(l, font))
            .collect()
    }

    fn line_width(line: &str, font: PxScaleFont<&FontVec>) -> f32 {
        let mut previous_glyph: Option<Glyph> = None;
        line.chars()
            .filter(|c| !c.is_control())
            .map(|c| {
                let glyph = font.scaled_glyph(c);
                let width = font.h_advance(glyph.id)
                    + previous_glyph
                        .as_ref()
                        .map_or(0., |g| font.kern(g.id, glyph.id));
                previous_glyph = Some(glyph);
                width
            })
            .sum::<f32>()
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn height(&self, font: PxScaleFont<&FontVec>) -> u32 {
        let line_count = self.content.lines().count() + usize::from(self.content.ends_with('\n'));
        let gap_count = line_count.saturating_sub(1);
        let height = font.height().mul_add(
            line_count as f32,
            font.line_gap().mul_add(gap_count as f32, 1.),
        );
        height.ceil() as u32
    }

    fn render_glyphs(
        &self,
        font: PxScaleFont<&FontVec>,
        width: f32,
        line_widths: &[f32],
        buffer: &mut [u8],
        size: Size,
    ) {
        let v_advance = font.height() + font.line_gap();
        let mut cursor_y = font.ascent();
        for (line, &line_width) in self.content.lines().zip(line_widths) {
            let mut cursor_x = match self.alignment {
                Alignment::Left => 0.,
                Alignment::Center => (width - line_width) / 2.,
                Alignment::Right => width - line_width,
            };
            let mut previous_glyph_id = None;
            for character in line.chars().filter(|c| !c.is_control()) {
                let mut glyph = font.scaled_glyph(character);
                glyph.position = ab_glyph::point(cursor_x, cursor_y);
                cursor_x += font.h_advance(glyph.id);
                if let Some(last_glyph_id) = previous_glyph_id {
                    cursor_x += font.kern(last_glyph_id, glyph.id);
                }
                previous_glyph_id = Some(glyph.id);
                Self::render_glyph(font, glyph, buffer, size);
            }
            cursor_y += v_advance;
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn render_glyph(font: PxScaleFont<&FontVec>, glyph: Glyph, buffer: &mut [u8], size: Size) {
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, v| {
                let x = x + bounds.min.x as u32 + TEXTURE_PADDING_PX;
                let y = y + bounds.min.y as u32 + TEXTURE_PADDING_PX;
                let idx = (y * size.width + x) as usize * 4;
                buffer[idx] = 255;
                buffer[idx + 1] = 255;
                buffer[idx + 2] = 255;
                buffer[idx + 3] = buffer[idx + 3].saturating_add((v * 255.) as u8);
            });
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Alignment {
    #[default]
    Center,
    Left,
    Right,
}