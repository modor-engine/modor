use super::resources::fonts::{FontKey, FontStorage};
use super::resources::textures::TextureStorage;
use super::texts::TextStorage;
use crate::backend::data::{Camera, Instance};
use crate::backend::renderer::Renderer;
use crate::backend::rendering::{RenderCommands, Rendering};
use crate::backend::textures::Image;
use crate::backend::uniforms::Uniform;
use crate::storages::models::ModelStorage;
use crate::storages::opaque_instances::OpaqueInstanceStorage;
use crate::storages::shaders::ShaderStorage;
use crate::storages::transparent_instances::TransparentInstanceStorage;
use crate::utils::numbers;
use crate::{
    Alignment, Color, Font, InternalTextureConfig, Mesh2D, Shape, SurfaceSize, Text2D, TextSize,
    Texture, TexturePart,
};
use ab_glyph::FontVec;
use modor::Query;
use modor_math::{Mat4, Quat, Vec2, Vec3};
use modor_physics::Transform2D;
use winit::window::Window;

const MAX_DEPTH: f32 = 0.9; // used to fix shape disappearance when depth is near to 1

pub(crate) type ShapeComponents<'a> = (&'a Transform2D, &'a Mesh2D);
pub(crate) type TextComponents<'a> = (&'a Transform2D, &'a mut Text2D);

pub(crate) struct CoreStorage {
    renderer: Renderer,
    camera_2d: Uniform<Camera>,
    shaders: ShaderStorage,
    models: ModelStorage,
    textures: TextureStorage,
    fonts: FontStorage,
    opaque_instances: OpaqueInstanceStorage,
    transparent_instances: TransparentInstanceStorage,
    texts: TextStorage,
}

impl CoreStorage {
    pub(crate) fn new(renderer: Renderer) -> Self {
        let camera_2d = Uniform::new(vec![Camera::default()], 0, "camera_2d", &renderer);
        Self {
            shaders: ShaderStorage::new(&camera_2d, &renderer),
            camera_2d,
            models: ModelStorage::new(&renderer),
            textures: TextureStorage::new(&renderer),
            fonts: FontStorage::new(),
            opaque_instances: OpaqueInstanceStorage::default(),
            transparent_instances: TransparentInstanceStorage::new(&renderer),
            renderer,
            texts: TextStorage::default(),
        }
    }

    pub(crate) fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    // coverage: off (no surface refresh with capture)
    pub(crate) fn refresh_surface(&mut self, window: &Window) {
        self.renderer.refresh_surface(window);
    }
    // coverage: on

    pub(crate) fn set_size(&mut self, size: SurfaceSize) {
        self.renderer.set_size(size.width, size.height);
    }

    pub(crate) fn toggle_vsync(&mut self, enabled: bool) {
        self.renderer.toggle_vsync(enabled);
    }

    pub(crate) fn load_texture(&mut self, image: Image, config: &InternalTextureConfig) {
        self.textures.load(image, config, true, &self.renderer);
    }

    pub(crate) fn load_font(&mut self, key: FontKey, font: FontVec) {
        self.fonts.load(key, font);
    }

    pub(crate) fn remove_not_found_resources(
        &mut self,
        textures: &Query<'_, &Texture>,
        fonts: &Query<'_, &Font>,
    ) {
        let texture_keys = textures
            .iter()
            .map(|t| &t.config.key)
            .chain(self.texts.texture_keys());
        self.textures.remove_not_found(texture_keys);
        self.fonts.remove_not_found(fonts.iter().map(|t| &t.key));
    }

    pub(crate) fn update_instances(
        &mut self,
        shapes: Query<'_, ShapeComponents<'_>>,
        mut texts: Query<'_, TextComponents<'_>>,
        camera_transform: &Transform2D,
    ) {
        self.opaque_instances.reset();
        self.transparent_instances.reset();
        self.texts.reset();
        let depth_bounds = Self::depth_bounds(
            shapes
                .iter()
                .map(|(_, m)| m.z)
                .chain(texts.iter().map(|(_, t)| t.z)),
        );
        for (transform, mesh) in shapes.iter() {
            let has_texture = self.has_mesh_texture(mesh);
            let color = Self::mesh_color(mesh, has_texture);
            if color.a <= 0. {
                continue;
            }
            self.update_instance(transform, mesh, depth_bounds, color, has_texture);
        }
        for (transform, text) in texts.iter_mut() {
            let (texture_key, texture_size) =
                self.texts
                    .register(text, &self.fonts, &mut self.textures, &mut self.renderer);
            if text.color.a <= 0. {
                continue;
            }
            let mesh = Self::texture_mesh(text, texture_size, transform, texture_key);
            self.update_instance(transform, &mesh, depth_bounds, text.color, true);
        }
        self.transparent_instances.sort();
        self.camera_2d.buffer_mut().data_mut()[0] =
            Self::create_camera_data(camera_transform, &self.renderer);
        self.texts.delete_unregistered();
    }

    pub(crate) fn render(&mut self, background_color: Color) {
        self.camera_2d.buffer_mut().sync(&self.renderer);
        self.opaque_instances.sync_buffers(&self.renderer);
        self.transparent_instances.sync_buffers(&self.renderer);
        let mut rendering = Rendering::new(&mut self.renderer);
        {
            let mut commands = RenderCommands::new(background_color.into(), &mut rendering);
            commands.push_uniform_binding(&self.camera_2d, 0);
            self.opaque_instances.render(
                &mut commands,
                &self.shaders,
                &self.textures,
                &self.models,
            );
            self.transparent_instances.render(
                &mut commands,
                &self.shaders,
                &self.textures,
                &self.models,
            );
        }
        rendering.apply();
    }

    fn update_instance(
        &mut self,
        transform: &Transform2D,
        mesh: &Mesh2D,
        depth_bounds: (f32, f32),
        color: Color,
        has_texture: bool,
    ) {
        let instance = Self::create_instance(transform, mesh, depth_bounds, color, has_texture);
        let shader_idx = ShaderStorage::idx(mesh);
        let model_idx = ModelStorage::idx(mesh);
        let texture_key = mesh
            .texture_key
            .as_ref()
            .unwrap_or_else(|| self.textures.default_key());
        if color.a < 1. || self.textures.is_transparent(texture_key) {
            self.transparent_instances
                .add(instance, shader_idx, texture_key.clone(), model_idx);
        } else {
            self.opaque_instances.add(
                instance,
                shader_idx,
                texture_key.clone(),
                model_idx,
                &self.renderer,
            );
        }
    }

    fn has_mesh_texture(&self, mesh: &Mesh2D) -> bool {
        mesh.texture_key
            .as_ref()
            .map_or(false, |k| self.textures.get(k).is_some())
    }

    fn mesh_color(mesh: &Mesh2D, has_texture: bool) -> Color {
        if has_texture {
            mesh.texture_color
        } else {
            mesh.color
        }
    }

    fn depth_bounds<I>(depths: I) -> (f32, f32)
    where
        I: Iterator<Item = f32>,
    {
        depths.fold((f32::INFINITY, 0.0_f32), |(min, max), b| {
            (min.min(b), max.max(b))
        })
    }

    fn texture_mesh(
        text: &mut Text2D,
        texture_size: Vec2,
        transform: &Transform2D,
        texture_key: super::resources::textures::TextureKey,
    ) -> Mesh2D {
        let texture_size = Self::text_texture_size(text, texture_size, transform);
        Mesh2D {
            color: Color::INVISIBLE,
            z: text.z,
            texture_color: text.color,
            texture_part: TexturePart {
                position: Self::text_texture_position(text, texture_size),
                size: texture_size,
            },
            texture_key: Some(texture_key),
            shape: Shape::Rectangle,
        }
    }

    fn text_texture_position(text: &mut Text2D, texture_size: Vec2) -> Vec2 {
        Vec2::new(
            match text.alignment {
                Alignment::Left | Alignment::TopLeft | Alignment::BottomLeft => 0.,
                Alignment::Center | Alignment::Top | Alignment::Bottom => {
                    (1. - texture_size.x) / 2.
                }
                Alignment::Right | Alignment::TopRight | Alignment::BottomRight => {
                    1. - texture_size.x
                }
            },
            match text.alignment {
                Alignment::TopLeft | Alignment::Top | Alignment::TopRight => 0.,
                Alignment::Left | Alignment::Center | Alignment::Right => {
                    (1. - texture_size.y) / 2.
                }
                Alignment::BottomLeft | Alignment::Bottom | Alignment::BottomRight => {
                    1. - texture_size.y
                }
            },
        )
    }

    #[allow(clippy::cast_precision_loss)]
    fn text_texture_size(text: &mut Text2D, texture_size: Vec2, transform: &Transform2D) -> Vec2 {
        let line_count = text.string.lines().count() + usize::from(text.string.ends_with('\n'));
        let texture_ratio = texture_size.x / texture_size.y;
        let transform_ratio = transform.size.x / transform.size.y;
        match text.size {
            TextSize::Auto => Vec2::new(
                (transform_ratio / texture_ratio).max(1.),
                (texture_ratio / transform_ratio).max(1.),
            ),
            TextSize::LineHeight(line_height) => Vec2::new(
                transform.size.y * transform_ratio
                    / texture_ratio
                    / line_height
                    / line_count as f32,
                transform.size.y / line_height / line_count as f32,
            ),
        }
    }

    fn create_instance(
        transform: &Transform2D,
        mesh: &Mesh2D,
        depth_bounds: (f32, f32),
        color: Color,
        has_texture: bool,
    ) -> Instance {
        let (min_z, max_z) = depth_bounds;
        let z = MAX_DEPTH - numbers::normalize(mesh.z, min_z, max_z, 0., MAX_DEPTH);
        let matrix = Mat4::from_scale(transform.size.with_z(0.))
            * Quat::from_z(*transform.rotation).matrix()
            * Mat4::from_position(transform.position.with_z(z));
        Instance {
            transform: matrix.to_array(),
            color: color.into(),
            has_texture: has_texture.into(),
            texture_part_position: [mesh.texture_part.position.x, mesh.texture_part.position.y],
            texture_part_size: [mesh.texture_part.size.x, mesh.texture_part.size.y],
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn create_camera_data(camera_transform: &Transform2D, renderer: &Renderer) -> Camera {
        let size = renderer.target_size();
        let (x_scale, y_scale) = numbers::world_scale(size);
        let position = Vec3::from_xy(-camera_transform.position.x, -camera_transform.position.y);
        let scale = Vec3::new(
            2. * x_scale / camera_transform.size.x,
            2. * y_scale / camera_transform.size.y,
            1.,
        );
        Camera {
            transform: (Mat4::from_position(position)
                * Quat::from_z(*camera_transform.rotation).matrix()
                * Mat4::from_scale(scale))
            .to_array(),
        }
    }
}
