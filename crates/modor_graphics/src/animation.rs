use instant::Instant;
use modor::{App, Builder};
use modor_input::modor_math::Vec2;
use std::time::Duration;

/// A utility type for handling texture animation.
///
/// This type helps to change at regular interval the part of a texture used by a material.
///
/// It is expected that:
/// - The texture is a grid of texture sub-parts (e.g. a spritesheet).
/// - Each texture part has the same size.
/// - The size of the texture is a multiple of the size of the texture parts.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// #
/// struct AnimatedSprite {
///     sprite: Sprite2D,
///     animation: TextureAnimation,
///     texture: Res<Texture>
/// }
///
/// impl AnimatedSprite {
///     fn new(app: &mut App) -> Self {
///         let texture = Texture::new(app).load_from_path(app, "spritesheet.png");
///         let animation_parts = vec![
///             TexturePart::new(0, 0),
///             TexturePart::new(1, 0),
///             TexturePart::new(2, 0),
///             TexturePart::new(0, 1),
///             TexturePart::new(1, 1),
///             TexturePart::new(2, 1),
///         ];
///         Self {
///             sprite: Sprite2D::new(app)
///                 .with_material(|m| m.texture = texture.glob().to_ref()),
///             animation: TextureAnimation::new(3, 2)
///                 .with_fps(5)
///                 .with_parts(|p| *p = animation_parts),
///             texture,
///         }
///     }
///
///     fn update(&mut self, app: &mut App) {
///         self.sprite.material.texture_size = self.animation.part_size();
///         self.sprite.material.texture_position = self.animation.part_position();
///         self.sprite.update(app);
///         self.animation.update(app);
///         self.texture.update(app);
///     }
/// }
/// ```
#[derive(Builder)]
pub struct TextureAnimation {
    /// The number of columns in the texture.
    ///
    /// The width of a texture part in pixels is the width of the texture in pixels divided by the
    /// number of columns.
    pub columns: u16,
    /// The number of lines in the texture.
    ///
    /// The height of a texture part in pixels is the height of the texture in pixels divided by the
    /// number of lines.
    pub lines: u16,
    /// The number of frames per second of the animation.
    ///
    /// If equal to zero, then the first frame is always displayed.
    ///
    /// Default value is 10.
    #[builder(form(value))]
    pub fps: u16,
    /// The successive texture parts displayed in loop during the animation.
    ///
    /// If empty, then the top-left texture part is always displayed.
    ///
    /// Default is empty.
    #[builder(form(closure))]
    pub parts: Vec<TexturePart>,
    last_update_instant: Instant,
    current_part_index: Option<usize>,
}

impl TextureAnimation {
    const DEFAULT_FPS: u16 = 10;

    /// Creates a new animation.
    pub fn new(columns: u16, lines: u16) -> Self {
        Self {
            columns,
            lines,
            fps: Self::DEFAULT_FPS,
            parts: vec![],
            last_update_instant: Instant::now(),
            current_part_index: None,
        }
    }

    /// Updates the animation.
    pub fn update(&mut self, _app: &mut App) {
        if let Some(new_frame_elapsed_time) = self
            .last_update_instant
            .elapsed()
            .checked_sub(self.frame_duration())
        {
            let now = Instant::now();
            self.last_update_instant = now.checked_sub(new_frame_elapsed_time).unwrap_or(now);
            self.current_part_index = self.next_part_index();
        }
    }

    /// Returns the size of a texture part.
    ///
    /// The returned size has both components between `0.0` and `1.0`.
    pub fn part_size(&self) -> Vec2 {
        Vec2::new(
            1. / f32::from(self.columns.max(1)),
            1. / f32::from(self.lines.max(1)),
        )
    }

    /// Returns the top-left position of the current texture part.
    ///
    /// The returned position has both components between `0.0` and `1.0`.
    pub fn part_position(&self) -> Vec2 {
        if let Some(part) = self.current_part_index.and_then(|i| self.parts.get(i)) {
            self.part_size()
                .with_scale(Vec2::new(part.column.into(), part.line.into()))
        } else {
            Vec2::ZERO
        }
    }

    fn frame_duration(&self) -> Duration {
        if self.current_part_index.is_none() {
            Duration::ZERO
        } else if self.fps == 0 {
            Duration::MAX
        } else {
            Duration::from_secs_f32(1. / f32::from(self.fps))
        }
    }

    fn next_part_index(&self) -> Option<usize> {
        if self.parts.is_empty() {
            None
        } else if let Some(current_part_idx) = self.current_part_index {
            if current_part_idx < self.parts.len() - 1 {
                Some(current_part_idx + 1)
            } else {
                Some(0)
            }
        } else {
            Some(0)
        }
    }
}

/// The coordinates of a texture part.
///
/// This is used to define the successive texture parts displayed by [`TextureAnimation`].
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[non_exhaustive]
pub struct TexturePart {
    /// The column index inside the texture.
    pub column: u16,
    /// The line index inside the texture.
    pub line: u16,
}

impl TexturePart {
    /// Creates a new texture part configuration.
    pub const fn new(column: u16, line: u16) -> Self {
        Self { column, line }
    }
}
