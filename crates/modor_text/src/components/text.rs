use crate::components::font::{FontRegistry, DEFAULT_FONT};
use crate::Font;
use ab_glyph::{Font as AbFont, FontVec, Glyph, PxScaleFont, ScaleFont};
use modor::Custom;
use modor_graphics::{Size, Texture, TextureSource};
use modor_resources::{ResKey, ResourceAccessor};
use std::iter;

const TEXTURE_PADDING_PX: u32 = 1;

/// A text to render in a [`Texture`].
///
/// The size of the generated texture is calculated to exactly fit the text.
///
/// # Requirements
///
/// - text [`module`](crate::module()) is initialized
/// - [`Texture`] component is in the same entity
///
/// # Related components
///
/// - [`Texture`]
/// - [`Font`]
///
/// # Entity functions creating this component
///
/// - [`text_2d`](crate::text_2d())
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_physics::*;
/// # use modor_text::*;
/// # use modor_resources::*;
/// #
/// const FONT: ResKey<Font> = ResKey::new("custom");
///
/// fn root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .child_component(Font::from_path(FONT, "font.ttf"))
///         .child_entity(text())
/// }
///
/// fn text() -> impl BuiltEntity {
///     let texture_key = ResKey::unique("text");
///     instance_2d(WINDOW_CAMERA_2D, Default2DMaterial::new())
///         .updated(|m: &mut Default2DMaterial| m.front_texture_key = Some(texture_key))
///         .component(Text::new("my text", 30.))
///         .with(|t| t.font_key = FONT)
///         .component(Texture::from_size(texture_key, Size::ZERO))
/// }
/// ```
#[derive(Component, Debug)]
pub struct Text {
    /// Text to render.
    pub content: String,
    /// Font height of the rendered text.
    pub font_height: f32,
    /// Key of the [`Font`] used to render the text.
    ///
    /// If the font is not loaded, then the text is not rendered in the [`Texture`].
    ///
    /// Default is [Roboto](https://fonts.google.com/specimen/Roboto).
    pub font_key: ResKey<Font>,
    /// Alignment of the rendered text.
    ///
    /// Default is [`Alignment::Center`].
    pub alignment: Alignment,
    old_content: String,
    old_font_height: f32,
    old_font_key: ResKey<Font>,
    old_alignment: Alignment,
}

#[systems]
impl Text {
    /// Creates a new text with a given `content` and `font_height`.
    pub fn new(content: impl Into<String>, font_height: f32) -> Self {
        Self {
            content: content.into(),
            font_height,
            font_key: DEFAULT_FONT,
            alignment: Alignment::default(),
            old_content: String::new(),
            old_font_height: font_height,
            old_font_key: DEFAULT_FONT,
            old_alignment: Alignment::default(),
        }
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    #[run_after(component(FontRegistry), component(Font))]
    fn update(&mut self, texture: &mut Texture, fonts: Custom<ResourceAccessor<'_, Font>>) {
        if let Some(font) = fonts.get(self.font_key) {
            if self.has_changed() || font.is_just_loaded {
                let font = font.get().as_scaled(self.font_height);
                let line_widths = self.line_widths(font);
                let width = line_widths.iter().fold(0.0_f32, |a, &b| a.max(b)).max(1.);
                let height = self.height(font).max(1);
                let size = Size::new(
                    width.ceil() as u32 + TEXTURE_PADDING_PX * 2 + 2,
                    height + TEXTURE_PADDING_PX * 2 + 2,
                );
                let mut buffer: Vec<_> = iter::repeat([255, 255, 255, 0])
                    .take((size.width * size.height) as usize)
                    .flatten()
                    .collect();
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
        self.old_font_key = self.font_key;
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
                let x = x + bounds.min.x as u32 + TEXTURE_PADDING_PX + 1;
                let y = y + bounds.min.y as u32 + TEXTURE_PADDING_PX + 1;
                let idx = (y * size.width + x) as usize * 4;
                buffer[idx] = 255;
                buffer[idx + 1] = 255;
                buffer[idx + 2] = 255;
                buffer[idx + 3] = buffer[idx + 3].saturating_add((v * 255.) as u8);
            });
        }
    }
}

/// The alignment of a rendered text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Alignment {
    /// Center alignment.
    #[default]
    Center,
    /// Left alignment.
    Left,
    /// Right alignment.
    Right,
}
