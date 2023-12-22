use crate::components::material::private::Text2DMaterialData;
use crate::entities::module::TEXT_SHADER;
use modor_graphics::{Color, MaterialSource, NoInstanceData, Shader, Texture};
use modor_resources::ResKey;

/// A material configuration for rendering 2D texts.
///
/// The text is centered and its aspect ratio is preserved.
///
/// # Requirements
///
/// The material is effective only if:
/// - text [`module`](crate::module()) is initialized
/// - the entity contains components of type [`Material`](modor_graphics::Material)
///     and [`MaterialSync<Text2DMaterial>`](modor_graphics::MaterialSync)
///
/// # Related components
///
/// - [`Material`](modor_graphics::Material)
/// - [`MaterialSync`](modor_graphics::MaterialSync)
/// - [`Text`](crate::Text)
///
/// # Entity functions creating this component
///
/// - [`text_2d`](crate::text_2d())
///
/// # Examples
///
/// See [`text_2d`](crate::text_2d()).
#[derive(Debug, Component, NoSystem)]
pub struct Text2DMaterial {
    /// Color of the rendered text.
    ///
    /// Default is [`Color::WHITE`].
    pub color: Color,
    /// Key of the texture containing the text to render.
    ///
    /// The text in the texture should be white.
    pub texture_key: ResKey<Texture>,
}

impl Text2DMaterial {
    /// Creates a new text material.
    pub fn new(texture_key: ResKey<Texture>) -> Self {
        Self {
            color: Color::WHITE,
            texture_key,
        }
    }
}

impl MaterialSource for Text2DMaterial {
    type Data = Text2DMaterialData;
    type InstanceData = NoInstanceData;

    fn data(&self) -> Self::Data {
        Text2DMaterialData {
            color: self.color.into(),
        }
    }

    fn texture_keys(&self) -> Vec<ResKey<Texture>> {
        vec![self.texture_key]
    }

    fn shader_key(&self) -> ResKey<Shader> {
        TEXT_SHADER
    }

    fn is_transparent(&self) -> bool {
        self.color.a > 0. && self.color.a < 1.
    }
}

mod private {
    use bytemuck::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Clone, Copy, Zeroable, Pod)]
    pub struct Text2DMaterialData {
        pub(super) color: [f32; 4],
    }
}
