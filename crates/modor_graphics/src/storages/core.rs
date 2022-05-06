use crate::backend::data::Instance;
use crate::backend::renderer::Renderer;
use crate::backend::rendering::{RenderCommands, Rendering};
use crate::storages::models::ModelStorage;
use crate::storages::opaque_instances::OpaqueInstanceStorage;
use crate::storages::shaders::ShaderStorage;
use crate::storages::transparent_instances::TransparentInstanceStorage;
use crate::{utils, Color, ShapeColor, SurfaceSize};
use modor::Query;
use modor_physics::{AbsolutePosition, Position, Scale, Shape, Size};

const MAX_DEPTH: f32 = 0.9; // used to fix shape disappearance when depth is near to 1

pub(crate) struct CoreStorage {
    renderer: Renderer,
    shaders: ShaderStorage,
    models: ModelStorage,
    opaque_instances: OpaqueInstanceStorage,
    transparent_instances: TransparentInstanceStorage,
}

impl CoreStorage {
    pub(crate) fn new(renderer: Renderer) -> Self {
        Self {
            shaders: ShaderStorage::new(&renderer),
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
        no_mutation!(self.renderer.toggle_vsync(enabled)); // window cannot be tested
    }

    pub(crate) fn update_instances(
        &mut self,
        shapes: Query<'_, (&ShapeColor, &Position, &Scale, Option<&Shape>)>,
    ) {
        self.opaque_instances.reset();
        self.transparent_instances.reset();
        let fixed_scale = self.fixed_scale();
        let depth_bounds = Self::depth_bounds(shapes.iter().map(|(_, p, _, _)| p.z));
        for (color, position, scale, shape) in shapes.iter() {
            let instance = Self::create_instance(
                color,
                position.abs(),
                scale.abs(),
                depth_bounds,
                fixed_scale,
            );
            let shape = shape.unwrap_or(&Shape::Rectangle2D);
            let shader_idx = self.shaders.idx(shape);
            let model_idx = self.models.idx(shape);
            if Self::is_transparent(color.0.a) {
                self.transparent_instances
                    .add(instance, shader_idx, model_idx);
            } else {
                self.opaque_instances
                    .add(instance, shader_idx, model_idx, &self.renderer);
            }
        }
        self.transparent_instances.sort();
    }

    pub(crate) fn render(&mut self, background_color: Color) {
        self.opaque_instances.sync_buffers(&self.renderer);
        self.transparent_instances.sync_buffers(&self.renderer);
        let mut rendering = Rendering::new(&mut self.renderer);
        {
            let mut commands = RenderCommands::new(background_color.into(), &mut rendering);
            self.opaque_instances
                .render(&mut commands, &self.shaders, &self.models);
            self.transparent_instances
                .render(&mut commands, &self.shaders, &self.models);
        }
        rendering.apply();
    }

    #[allow(clippy::cast_precision_loss)]
    fn fixed_scale(&self) -> (f32, f32) {
        let size = self.renderer.target_size();
        let x_scale = f32::min(size.1 as f32 / size.0 as f32, 1.);
        let y_scale = f32::min(size.0 as f32 / size.1 as f32, 1.);
        (x_scale, y_scale)
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
        color: &ShapeColor,
        position: &AbsolutePosition,
        size: &Size,
        depth_bounds: (f32, f32),
        fixed_scale: (f32, f32),
    ) -> Instance {
        let (min_z, max_z) = depth_bounds;
        let (x_scale, y_scale) = fixed_scale;
        let z_position = MAX_DEPTH - utils::normalize(position.z, min_z, max_z, 0., MAX_DEPTH);
        Instance {
            transform: [
                [size.x * 2. * x_scale, 0., 0., 0.],
                [0., size.y * 2. * y_scale, 0., 0.],
                [0., 0., 0., 0.],
                [
                    position.x * 2. * x_scale,
                    position.y * 2. * y_scale,
                    z_position,
                    1.,
                ],
            ],
            color: [color.0.r, color.0.g, color.0.b, color.0.a],
        }
    }

    // coverage: off (optimization that cannot pass mutation tests)
    fn is_transparent(transparency: f32) -> bool {
        transparency > 0. && transparency < 1.
    }
    // coverage: on
}
