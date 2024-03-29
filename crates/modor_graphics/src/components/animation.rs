use crate::MaterialSource;
use instant::Instant;
use modor::{VariableSend, VariableSync};
use modor_math::Vec2;
use std::marker::PhantomData;
use std::time::Duration;

/// A animation configuration of a material texture of type `M`.
///
/// This component helps to change at regular interval the part of a texture used by a material.
///
/// It is expected that:
/// - The texture is a spritesheet, i.e. a grid of sprites.
/// - Each sprite has the same size.
/// - The size of the texture is a multiple of the size of the sprites.
///
/// # Requirements
///
/// The texture can be loaded only if:
/// - graphics [`module`](crate::module()) is initialized
/// - material of type `M` is in the same entity
///
/// # Related components
///
/// - [`Material`](crate::Material)
/// - [`Texture`](crate::Texture)
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_physics::*;
/// # use modor_math::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// #
/// fn root() -> impl BuiltEntity {
///     let texture_key = ResKey::unique("spritesheet");
///     EntityBuilder::new()
///         .component(Texture::from_path(texture_key, "spritesheet.png"))
///         .child_entity(animated_instance(texture_key))
/// }
///
/// fn animated_instance(texture_key : ResKey<Texture>) -> impl BuiltEntity {
///     let sprites = vec![
///         Sprite::new(0, 0),
///         Sprite::new(1, 0),
///         Sprite::new(2, 0),
///         Sprite::new(0, 1),
///         Sprite::new(1, 1),
///         Sprite::new(2, 1),
///     ];
///     instance_2d(WINDOW_CAMERA_2D, Default2DMaterial::new())
///         .updated(|m: &mut Default2DMaterial| m.texture_key = Some(texture_key))
///         .component(TextureAnimation::<Default2DMaterial>::new(3, 2, sprites))
///         .with(|a| a.frames_per_second = 5)
/// }
/// ```
#[derive(Component, Debug, Clone)]
pub struct TextureAnimation<M: 'static + VariableSync + VariableSend> {
    /// The number of columns in the texture.
    ///
    /// The width of a sprite in pixels is the width of the texture in pixels divided by the
    /// number of columns.
    pub columns: u16,
    /// The number of lines in the texture.
    ///
    /// The height of a sprite in pixels is the height of the texture in pixels divided by the
    /// number of lines.
    pub lines: u16,
    /// The number of frames per second of the animation.
    ///
    /// If equal to zero, then the first frame is always displayed.
    ///
    /// Default value is 10.
    pub frames_per_second: u16,
    /// The successive sprites displayed in loop during the animation.
    pub sprites: Vec<Sprite>,
    last_update_instant: Instant,
    current_sprite_idx: Option<usize>,
    phantom: PhantomData<M>,
}

#[systems]
impl<M> TextureAnimation<M>
where
    M: AnimatedMaterialSource,
{
    const DEFAULT_FRAMES_PER_SECOND: u16 = 10;

    /// Creates a new animation.
    #[allow(clippy::unchecked_duration_subtraction)]
    pub fn new(columns: u16, lines: u16, sprites: impl Into<Vec<Sprite>>) -> Self {
        Self {
            columns,
            lines,
            frames_per_second: Self::DEFAULT_FRAMES_PER_SECOND,
            sprites: sprites.into(),
            last_update_instant: Instant::now(),
            current_sprite_idx: None,
            phantom: PhantomData,
        }
    }

    #[run]
    fn update_material(&mut self, material: &mut M) {
        if self.last_update_instant.elapsed() >= self.frame_duration() {
            if let Some(sprite_idx) = self.next_sprite_idx() {
                let sprite = self.sprites[sprite_idx];
                let sprite_size =
                    Vec2::new(1. / f32::from(self.columns), 1. / f32::from(self.lines));
                let sprite_position =
                    sprite_size.with_scale(Vec2::new(sprite.column.into(), sprite.line.into()));
                material.update(sprite_size, sprite_position);
                self.current_sprite_idx = Some(sprite_idx);
            }
            self.last_update_instant = Instant::now();
        }
    }

    /// Returns the index of the current displayed sprite.
    pub fn current_sprite_index(&self) -> usize {
        self.current_sprite_idx.unwrap_or(0)
    }

    fn frame_duration(&self) -> Duration {
        if self.current_sprite_idx.is_none() {
            Duration::ZERO
        } else if self.frames_per_second == 0 {
            Duration::MAX
        } else {
            Duration::from_secs_f32(1. / f32::from(self.frames_per_second))
        }
    }

    fn next_sprite_idx(&self) -> Option<usize> {
        if self.sprites.is_empty() {
            error!("`TextureAnimation` without sprite");
            None
        } else if let Some(current_sprite_idx) = self.current_sprite_idx {
            if current_sprite_idx < self.sprites.len() - 1 {
                Some(current_sprite_idx + 1)
            } else {
                Some(0)
            }
        } else {
            Some(0)
        }
    }
}

/// The configuration of a sprite inside a spritesheet.
///
/// This is used to define the successive sprites displayed by [`TextureAnimation`].
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Sprite {
    /// The column index inside the spritesheet.
    pub column: u16,
    /// The line index inside the spritesheet.
    pub line: u16,
}

impl Sprite {
    /// Creates a new sprite configuration.
    pub const fn new(column: u16, line: u16) -> Self {
        Self { column, line }
    }
}

/// A trait for defining a material supporting [`TextureAnimation`].
///
/// # Examples
///
/// See [`TextureAnimation`].
pub trait AnimatedMaterialSource: MaterialSource {
    /// Updates the material to display the `sprite`.
    ///
    /// `sprite_size` is the size of the sprite in texture distances
    /// (each coordinate between `0.` and `1.`).<br>
    /// `sprite_position` is the size of the sprite in texture coordinates
    /// (each coordinate between `0.` and `1.` from top-left corner of the texture).
    fn update(&mut self, sprite_size: Vec2, sprite_position: Vec2);
}
