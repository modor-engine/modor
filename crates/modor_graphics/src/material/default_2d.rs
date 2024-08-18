use crate::resources::Resources;
use crate::{Color, MatGlob, MatUpdater, Material, Model2DGlob, Texture};
use modor::{App, Glob, GlobRef, Updater};
use modor_input::modor_math::Vec2;
use modor_resources::Res;
use std::marker::PhantomData;

/// The default material for 2D rendering.
///
/// # Examples
///
/// See [`Model2D`](crate::Model2D).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod, Updater)]
pub struct DefaultMaterial2D {
    pub(crate) shader_color: [f32; 4],
    pub(crate) shader_texture_part_position: [f32; 2],
    pub(crate) shader_texture_part_size: [f32; 2],
    /// Color of the rendered instance.
    ///
    /// This color is multiplied to the [`texture`](DefaultMaterial2DUpdater::texture) pixel colors.
    ///
    /// Default is [`Color::WHITE`].
    #[updater(inner_type, field, for_field)]
    color: PhantomData<Color>,
    /// Texture used to render the models.
    ///
    /// If the texture is not loaded, then the instances attached to the material are not rendered.
    ///
    /// Default is a white texture.
    #[updater(inner_type, field, for_field)]
    texture: PhantomData<GlobRef<Res<Texture>>>,
    /// Top-left position of the extracted texture section.
    ///
    /// [`Vec2::ZERO`] corresponds to top-left corner, and [`Vec2::ONE`] corresponds to bottom-right
    /// corner of the texture.
    ///
    /// Default is [`Vec2::ZERO`].
    #[updater(inner_type, field, for_field)]
    texture_position: PhantomData<Vec2>,
    /// Size of the extracted texture section.
    ///
    /// [`Vec2::ONE`] corresponds to the entire texture.
    ///
    /// Default is [`Vec2::ONE`].
    #[updater(inner_type, field, for_field)]
    texture_size: PhantomData<Vec2>,
    /// Whether the instance is rendered as an ellipse.
    ///
    /// If `false`, then the instance is displayed as a rectangle.
    ///
    /// Default is `false`.
    #[updater(inner_type, field, for_field)]
    is_ellipse: PhantomData<bool>,
}

impl Default for DefaultMaterial2D {
    fn default() -> Self {
        Self {
            shader_color: Color::WHITE.into(),
            shader_texture_part_position: [0., 0.],
            shader_texture_part_size: [1., 1.],
            color: PhantomData,
            texture: PhantomData,
            texture_position: PhantomData,
            texture_size: PhantomData,
            is_ellipse: PhantomData,
        }
    }
}

impl Material for DefaultMaterial2D {
    type InstanceData = ();

    fn init(app: &mut App, glob: &MatGlob<Self>) {
        MatUpdater::default()
            .shader(app.get_mut::<Resources>().default_shader.to_ref())
            .textures(vec![app.get_mut::<Resources>().white_texture.to_ref()])
            .is_transparent(false)
            .apply(app, glob);
    }

    fn instance_data(_app: &mut App, _model: &Glob<Model2DGlob>) -> Self::InstanceData {}
}

impl DefaultMaterial2DUpdater<'_> {
    /// Runs the update.
    pub fn apply(mut self, app: &mut App, glob: &MatGlob<DefaultMaterial2D>) {
        let mut updater = MatUpdater::default();
        if let Some(texture) = self
            .texture
            .take_value(|| Self::retrieve_texture(app, glob))
        {
            updater = updater.textures(vec![texture]);
        }
        if let Some(is_ellipse) = self
            .is_ellipse
            .take_value(|| Self::retrieve_is_ellipse(app, glob))
        {
            updater = updater.shader(if is_ellipse {
                app.get_mut::<Resources>().ellipse_shader.to_ref()
            } else {
                app.get_mut::<Resources>().default_shader.to_ref()
            });
        }
        let mut data = glob.data(app);
        let mut is_data_modified = false;
        if let Some(color) = self.color.take_value(|| data.shader_color.into()) {
            data.shader_color = color.into();
            is_data_modified = true;
        }
        if let Some(texture_position) = self.texture_position.take_value(|| {
            Vec2::new(
                data.shader_texture_part_position[0],
                data.shader_texture_part_position[1],
            )
        }) {
            data.shader_texture_part_position = [texture_position.x, texture_position.y];
            is_data_modified = true;
        }
        if let Some(texture_size) = self.texture_size.take_value(|| {
            Vec2::new(
                data.shader_texture_part_size[0],
                data.shader_texture_part_size[1],
            )
        }) {
            data.shader_texture_part_size = [texture_size.x, texture_size.y];
            is_data_modified = true;
        }
        if is_data_modified {
            updater = updater
                .data(data)
                .is_transparent(data.shader_color[3] > 0. && data.shader_color[3] < 1.);
        }
        updater.apply(app, glob);
    }

    fn retrieve_texture(app: &mut App, glob: &MatGlob<DefaultMaterial2D>) -> GlobRef<Res<Texture>> {
        let texture = glob.get(app).textures().next().cloned();
        texture.unwrap_or_else(|| app.get_mut::<Resources>().white_texture.to_ref())
    }

    fn retrieve_is_ellipse(app: &mut App, glob: &MatGlob<DefaultMaterial2D>) -> bool {
        glob.get(app).shader().index() == app.get_mut::<Resources>().ellipse_shader.index()
    }
}
