use crate::resources::Resources;
use crate::texture::glob::TextureGlob;
use crate::{Color, Material, Model2DGlob, ShaderGlobRef};
use internal::DefaultMaterial2DData;
use modor::{App, Builder, GlobRef};
use modor_input::modor_math::Vec2;

/// The default material for 2D rendering.
///
/// # Examples
///
/// See [`Model2D`](crate::Model2D).
#[derive(Debug, Builder)]
pub struct DefaultMaterial2D {
    /// Color of the rendered instance.
    ///
    /// This color is multiplied to the [`texture`](#structfield.texture) pixel colors.
    ///
    /// Default is [`Color::WHITE`].
    #[builder(form(value))]
    pub color: Color,
    /// Texture used to render the models.
    ///
    /// If the texture is not loaded, then the instances attached to the material are not rendered.
    ///
    /// Default is a white texture.
    #[builder(form(value))]
    pub texture: GlobRef<TextureGlob>,
    /// Top-left position of the extracted texture section.
    ///
    /// [`Vec2::ZERO`] corresponds to top-left corner, and [`Vec2::ONE`] corresponds to bottom-right
    /// corner of the texture.
    ///
    /// Default is [`Vec2::ZERO`].
    #[builder(form(value))]
    pub texture_position: Vec2,
    /// Size of the extracted texture section.
    ///
    /// [`Vec2::ONE`] corresponds to the entire texture.
    ///
    /// Default is [`Vec2::ONE`].
    #[builder(form(value))]
    pub texture_size: Vec2,
    /// Whether the instance is rendered as an ellipse.
    ///
    /// If `false`, then the instance is displayed as a rectangle.
    ///
    /// Default is `false`.
    #[builder(form(value))]
    pub is_ellipse: bool,
    default_shader: ShaderGlobRef<Self>,
    ellipse_shader: ShaderGlobRef<Self>,
}

impl Material for DefaultMaterial2D {
    type Data = DefaultMaterial2DData;
    type InstanceData = ();

    fn shader(&self) -> ShaderGlobRef<Self> {
        if self.is_ellipse {
            self.ellipse_shader.clone()
        } else {
            self.default_shader.clone()
        }
    }

    fn textures(&self) -> Vec<GlobRef<TextureGlob>> {
        vec![self.texture.clone()]
    }

    fn is_transparent(&self) -> bool {
        self.color.a > 0. && self.color.a < 1.
    }

    fn data(&self) -> Self::Data {
        DefaultMaterial2DData {
            color: self.color.into(),
            texture_part_position: [self.texture_position.x, self.texture_position.y],
            texture_part_size: [self.texture_size.x, self.texture_size.y],
        }
    }

    fn instance_data(_app: &mut App, _model: &GlobRef<Model2DGlob>) -> Self::InstanceData {}
}

impl DefaultMaterial2D {
    /// Creates a new material.
    pub fn new(app: &mut App) -> Self {
        let resources = app.get_mut::<Resources>();
        Self {
            color: Color::WHITE,
            texture: resources.white_texture.glob().clone(),
            texture_position: Vec2::ZERO,
            texture_size: Vec2::ONE,
            is_ellipse: false,
            default_shader: resources.default_shader.glob(),
            ellipse_shader: resources.ellipse_shader.glob(),
        }
    }
}

pub(super) mod internal {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
    pub struct DefaultMaterial2DData {
        pub(crate) color: [f32; 4],
        pub(crate) texture_part_position: [f32; 2],
        pub(crate) texture_part_size: [f32; 2],
    }
}
