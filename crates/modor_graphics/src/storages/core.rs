use crate::backend::data::{Camera, Instance};
use crate::backend::renderer::Renderer;
use crate::backend::rendering::{RenderCommands, Rendering};
use crate::backend::uniforms::Uniform;
use crate::storages::models::ModelStorage;
use crate::storages::opaque_instances::OpaqueInstanceStorage;
use crate::storages::shaders::ShaderStorage;
use crate::storages::transparent_instances::TransparentInstanceStorage;
use crate::{utils, Color, ShapeColor, SurfaceSize};
use modor::Query;
use modor_physics::{Position, Shape, Size};

const MAX_DEPTH: f32 = 0.9; // used to fix shape disappearance when depth is near to 1

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
        shapes: Query<'_, (&ShapeColor, &Position, &Size, Option<&Shape>)>,
        camera: CameraProperties,
    ) {
        self.opaque_instances.reset();
        self.transparent_instances.reset();
        let depth_bounds = Self::depth_bounds(shapes.iter().map(|(_, p, _, _)| p.z));
        for (color, position, scale, shape) in shapes.iter() {
            let instance = Self::create_instance(*color, *position, *scale, depth_bounds);
            let shape = shape.unwrap_or(&Shape::Rectangle2D);
            let shader_idx = self.shaders.idx(shape);
            let model_idx = self.models.idx(shape);
            if color.0.a > 0. && color.0.a < 1. {
                self.transparent_instances
                    .add(instance, shader_idx, model_idx);
            } else {
                self.opaque_instances
                    .add(instance, shader_idx, model_idx, &self.renderer);
            }
        }
        self.transparent_instances.sort();
        self.camera_2d.buffer_mut().data_mut()[0] =
            Self::create_camera_data(camera, &self.renderer);
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

    fn create_instance(
        color: ShapeColor,
        position: Position,
        size: Size,
        depth_bounds: (f32, f32),
    ) -> Instance {
        let (min_z, max_z) = depth_bounds;
        let z_position = MAX_DEPTH - utils::normalize(position.z, min_z, max_z, 0., MAX_DEPTH);
        Instance {
            transform: [
                [size.x, 0., 0., 0.],
                [0., size.y, 0., 0.],
                [0., 0., 0., 0.],
                [position.x, position.y, z_position, 1.],
            ],
            color: [color.0.r, color.0.g, color.0.b, color.0.a],
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn create_camera_data(camera: CameraProperties, renderer: &Renderer) -> Camera {
        let size = renderer.target_size();
        let x_scale = f32::min(size.1 as f32 / size.0 as f32, 1.);
        let y_scale = f32::min(size.0 as f32 / size.1 as f32, 1.);
        Camera {
            transform: [
                [2. * x_scale / camera.size.x, 0., 0., 0.],
                [0., 2. * y_scale / camera.size.y, 0., 0.],
                [0., 0., 1., 0.],
                [
                    -camera.position.x * 2. * x_scale / camera.size.x,
                    -camera.position.y * 2. * y_scale / camera.size.y,
                    0.,
                    1.,
                ],
            ],
        }
    }
}

pub(crate) struct CameraProperties {
    pub(crate) position: Position,
    pub(crate) size: Size,
}
