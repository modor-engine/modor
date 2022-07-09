use crate::backend::data::{Camera, Instance};
use crate::backend::renderer::Renderer;
use crate::backend::rendering::{RenderCommands, Rendering};
use crate::backend::uniforms::Uniform;
use crate::storages::models::ModelStorage;
use crate::storages::opaque_instances::OpaqueInstanceStorage;
use crate::storages::shaders::ShaderStorage;
use crate::storages::transparent_instances::TransparentInstanceStorage;
use crate::{utils, Color, Mesh, SurfaceSize};
use modor::Query;
use modor_math::{Mat4, Vec3};
use modor_physics::Transform;

const MAX_DEPTH: f32 = 0.9; // used to fix shape disappearance when depth is near to 1

pub(crate) type ShapeComponents<'a> = (&'a Transform, &'a Mesh);

pub(crate) struct CoreStorage {
    renderer: Renderer,
    camera_2d: Uniform<Camera>,
    shaders: ShaderStorage,
    models: ModelStorage,
    opaque_instances: OpaqueInstanceStorage,
    transparent_instances: TransparentInstanceStorage,
}

impl CoreStorage {
    pub(crate) fn new(renderer: Renderer) -> Self {
        let camera_2d = Uniform::new(vec![Camera::default()], 0, "camera_2d", &renderer);
        Self {
            shaders: ShaderStorage::new(&camera_2d, &renderer),
            camera_2d,
            models: ModelStorage::new(&renderer),
            opaque_instances: OpaqueInstanceStorage::default(),
            transparent_instances: TransparentInstanceStorage::new(&renderer),
            renderer,
        }
    }

    pub(crate) fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub(crate) fn set_size(&mut self, size: SurfaceSize) {
        self.renderer.set_size(size.width, size.height);
    }

    pub(crate) fn toggle_vsync(&mut self, enabled: bool) {
        self.renderer.toggle_vsync(enabled);
    }

    pub(crate) fn update_instances(
        &mut self,
        shapes: Query<'_, ShapeComponents<'_>>,
        camera_transform: &Transform,
    ) {
        self.opaque_instances.reset();
        self.transparent_instances.reset();
        let depth_bounds = Self::depth_bounds(shapes.iter().map(|(t, _)| t.position.z));
        for (transform, mesh) in shapes.iter() {
            let instance = Self::create_instance(transform, mesh, depth_bounds);
            let shader_idx = ShaderStorage::idx(mesh);
            let model_idx = ModelStorage::idx(mesh);
            if mesh.color.a > 0. && mesh.color.a < 1. {
                self.transparent_instances
                    .add(instance, shader_idx, model_idx);
            } else {
                self.opaque_instances
                    .add(instance, shader_idx, model_idx, &self.renderer);
            }
        }
        self.transparent_instances.sort();
        self.camera_2d.buffer_mut().data_mut()[0] =
            Self::create_camera_data(camera_transform, &self.renderer);
    }

    pub(crate) fn render(&mut self, background_color: Color) {
        self.camera_2d.buffer_mut().sync(&self.renderer);
        self.opaque_instances.sync_buffers(&self.renderer);
        self.transparent_instances.sync_buffers(&self.renderer);
        let mut rendering = Rendering::new(&mut self.renderer);
        {
            let mut commands = RenderCommands::new(background_color.into(), &mut rendering);
            commands.push_uniform_binding(&self.camera_2d, 0);
            self.opaque_instances
                .render(&mut commands, &self.shaders, &self.models);
            self.transparent_instances
                .render(&mut commands, &self.shaders, &self.models);
        }
        rendering.apply();
    }

    fn depth_bounds<I>(depths: I) -> (f32, f32)
    where
        I: Iterator<Item = f32>,
    {
        depths.fold((f32::INFINITY, 0.0_f32), |(min, max), b| {
            (min.min(b), max.max(b))
        })
    }

    fn create_instance(transform: &Transform, mesh: &Mesh, depth_bounds: (f32, f32)) -> Instance {
        let (min_z, max_z) = depth_bounds;
        let z_position =
            MAX_DEPTH - utils::normalize(transform.position.z, min_z, max_z, 0., MAX_DEPTH);
        let mut transform = transform.clone();
        transform.position.z = z_position;
        Instance {
            transform: transform.create_matrix().to_array(),
            color: mesh.color.into(),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn create_camera_data(camera_transform: &Transform, renderer: &Renderer) -> Camera {
        let size = renderer.target_size();
        let (x_scale, y_scale) = utils::world_scale(size);
        let position = Vec3::xy(-camera_transform.position.x, -camera_transform.position.y);
        let scale = Vec3::xyz(
            2. * x_scale / camera_transform.size.x,
            2. * y_scale / camera_transform.size.y,
            1.,
        );
        Camera {
            transform: (Mat4::from_position(position)
                * camera_transform.rotation.matrix()
                * Mat4::from_scale(scale))
            .to_array(),
        }
    }
}
