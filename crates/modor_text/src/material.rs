use crate::resources::TextResources;
use modor::{App, Glob, GlobRef, Updater};
use modor_graphics::modor_resources::Res;
use modor_graphics::{Color, MatGlob, MatUpdater, Material, Model2DGlob, Texture};
use std::marker::PhantomData;

/// A material for 2D text rendering.
///
/// # Examples
///
/// See [`Text2D`](crate::Text2D).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod, Updater)]
pub struct TextMaterial2D {
    shader_color: [f32; 4],
    /// The color of the rendered text.
    ///
    /// Default is [`Color::WHITE`].
    #[updater(inner_type, field)]
    color: PhantomData<Color>,
    /// The texture containing the text the render.
    ///
    /// Default is a white texture.
    #[updater(inner_type, field)]
    texture: PhantomData<GlobRef<Res<Texture>>>,
}

impl Default for TextMaterial2D {
    fn default() -> Self {
        Self {
            shader_color: Color::WHITE.into(),
            color: PhantomData,
            texture: PhantomData,
        }
    }
}

impl Material for TextMaterial2D {
    type InstanceData = ();

    fn init(app: &mut App, glob: &MatGlob<Self>) {
        MatUpdater::default()
            .shader(app.get_mut::<TextResources>().text_shader.to_ref())
            .apply(app, glob);
    }

    fn instance_data(_app: &mut App, _model: &Glob<Model2DGlob>) -> Self::InstanceData {}
}

impl TextMaterial2DUpdater<'_> {
    /// Runs the update.
    pub fn apply(mut self, app: &mut App, glob: &MatGlob<TextMaterial2D>) {
        let mut updater = MatUpdater::default();
        if let Some(texture) = self.texture.take_value(|| unreachable!()) {
            updater = updater.textures(vec![texture]);
        }
        if let Some(color) = self.color.take_value(|| unreachable!()) {
            updater = updater
                .data(TextMaterial2D {
                    shader_color: color.into(),
                    color: PhantomData,
                    texture: PhantomData,
                })
                .is_transparent(color.a > 0. && color.a < 1.);
        }
        updater.apply(app, glob);
    }
}
