use crate::resources::TextResources;
use crate::{FontGlob, TextMaterial2D};
use ab_glyph::{Font, FontVec, Glyph, PxScaleFont, ScaleFont};
use modor::{App, Builder, GlobRef};
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::{IntoMat, Mat, Model2D, Size, Texture, TextureSource};
use std::iter;

/// A rendered 2D text.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_graphics::modor_resources::*;
/// # use modor_text::*;
/// #
/// struct Root {
///     text: Text2D,
/// }
///
/// impl FromApp for Root {
///     fn from_app(app: &mut App) -> Self {
///         let font = app.get_mut::<Resources>().font.glob().to_ref();
///         Self {
///             text: Text2D::new(app)
///                 .with_content("Hello world!".into())
///                 .with_font(font)
///                 .with_font_height(200.)
///                 .with_material(|m| m.color = Color::GREEN),
///         }
///     }
/// }
///
/// impl RootNode for Root {
///     fn update(&mut self, app: &mut App) {
///         self.text.update(app);
///     }
/// }
///
/// struct Resources {
///     font: Res<Font>,
/// }
///
/// impl FromApp for Resources {
///     fn from_app(app: &mut App) -> Self {
///         Self {
///             font: Font::new(app).load_from_path(app, "my-font.ttf"),
///         }
///     }
/// }
///
/// impl RootNode for Resources {
///     fn update(&mut self, app: &mut App) {
///         self.font.update(app);
///     }
/// }
/// ```
#[derive(Debug, Builder)]
#[non_exhaustive]
pub struct Text2D {
    /// Text to render.
    ///
    /// Default is an empty string.
    #[builder(form(value))]
    pub content: String,
    /// Font height of the rendered text.
    ///
    /// This impacts the resolution of the rendered text.
    ///
    /// Default is `100.0`.
    #[builder(form(value))]
    pub font_height: f32,
    /// Font used to render the text.
    ///
    /// If the font is not loaded, then the text is not rendered.
    ///
    /// Default is [Roboto](https://fonts.google.com/specimen/Roboto).
    #[builder(form(value))]
    pub font: GlobRef<FontGlob>,
    /// Alignment of the rendered text.
    ///
    /// Default is [`Alignment::Center`].
    #[builder(form(value))]
    pub alignment: Alignment,
    /// Texture of the rendered text.
    ///
    /// The size of the generated texture is calculated to exactly fit the text.
    #[builder(form(closure))]
    pub texture: Res<Texture>,
    /// Material of the rendered text.
    #[builder(form(closure))]
    pub material: Mat<TextMaterial2D>,
    /// Model of the rendered text.
    #[builder(form(closure))]
    pub model: Model2D<TextMaterial2D>,
    old_state: OldState,
}

impl Text2D {
    const TEXTURE_PADDING_PX: u32 = 1;

    /// Creates a new sprite.
    pub fn new(app: &mut App) -> Self {
        let font = app.get_mut::<TextResources>().default_font.glob().to_ref();
        let texture = Texture::new(app)
            .load_from_source(app, TextureSource::Buffer(Size::ONE, vec![0, 0, 0, 0]));
        let material = TextMaterial2D::new(app, texture.glob().to_ref()).into_mat(app);
        let model = Model2D::new(app, material.glob());
        Self {
            content: String::new(),
            font_height: 100.,
            font: font.clone(),
            alignment: Alignment::default(),
            texture,
            material,
            model,
            old_state: OldState::new(font),
        }
    }

    /// Updates the text.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn update(&mut self, app: &mut App) {
        let font = self.font.get(app);
        if let Some(font_vec) = &font.font {
            if self.old_state.has_changed(self) || font.has_changed {
                let scaled_font = font_vec.as_scaled(self.font_height);
                let line_widths = self.line_widths(scaled_font);
                let width = line_widths.iter().fold(0.0_f32, |a, &b| a.max(b)).max(1.);
                let height = self.height(scaled_font).max(1);
                let size = Size::new(
                    width.ceil() as u32 + (Self::TEXTURE_PADDING_PX + 1) * 2,
                    height + (Self::TEXTURE_PADDING_PX + 1) * 2,
                );
                let mut buffer: Vec<_> = iter::repeat([255, 255, 255, 0])
                    .take((size.width * size.height) as usize)
                    .flatten()
                    .collect();
                self.render_glyphs(scaled_font, width, &line_widths, &mut buffer, size);
                self.texture
                    .reload_with_source(TextureSource::Buffer(size, buffer));
                self.update_old_state();
            }
        }
        self.texture.update(app);
        self.material.update(app);
        self.model.update(app);
    }

    fn update_old_state(&mut self) {
        self.old_state.content.clone_from(&self.content);
        self.old_state.font_height = self.font_height;
        self.old_state.font = self.font.clone();
        self.old_state.alignment = self.alignment;
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
                let x = x + bounds.min.x as u32 + Self::TEXTURE_PADDING_PX + 1;
                let y = y + bounds.min.y as u32 + Self::TEXTURE_PADDING_PX + 1;
                if x < size.width && y < size.height {
                    let idx = (y * size.width + x) as usize * 4;
                    buffer[idx] = 255;
                    buffer[idx + 1] = 255;
                    buffer[idx + 2] = 255;
                    buffer[idx + 3] = buffer[idx + 3].saturating_add((v * 255.) as u8);
                }
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

#[derive(Debug)]
struct OldState {
    content: String,
    font_height: f32,
    font: GlobRef<FontGlob>,
    alignment: Alignment,
}

impl OldState {
    fn new(font: GlobRef<FontGlob>) -> Self {
        Self {
            content: String::new(),
            font_height: 100.,
            font,
            alignment: Alignment::default(),
        }
    }

    #[allow(clippy::float_cmp)]
    fn has_changed(&self, text: &Text2D) -> bool {
        self.font_height != text.font_height
            || self.alignment != text.alignment
            || self.font != text.font
            || self.content != text.content
    }
}
