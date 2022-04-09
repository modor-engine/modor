use crate::backend::data::Instance;
use crate::backend::renderer::{Renderer, TargetView};
use crate::backend::rendering::{RenderCommands, Rendering};
use crate::storages::models::ModelStorage;
use crate::storages::opaque_instances::OpaqueInstanceStorage;
use crate::storages::shaders::ShaderStorage;
use crate::storages::transparent_instances::TransparentInstanceStorage;
use crate::{Color, ShapeColor, SurfaceSize};
use modor::Query;
use modor_physics::{Position, Scale, Shape};

const DEFAULT_SCALE: Scale = Scale::xyz(1., 1., 1.);
const MAX_2D_DEPTH: f32 = 0.9; // used to fix shape disappearance when depth is near to 1

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

    pub(crate) fn target_view(&mut self) -> TargetView<'_> {
        self.renderer.target_view()
    }

    pub(crate) fn set_size(&mut self, size: SurfaceSize) {
        self.renderer.resize(size.width, size.height);
    }

    pub(crate) fn update_instances(
        &mut self,
        shapes: Query<'_, (&ShapeColor, &Position, Option<&Scale>, Option<&Shape>)>,
    ) {
        self.opaque_instances.reset();
        self.transparent_instances.reset();
        let fixed_scale = self.fixed_scale();
        let depth_bounds = Self::depth_bounds(shapes.iter().map(|(_, p, _, _)| p.z));
        for (color, position, scale, shape) in shapes.iter() {
            let instance = Self::create_instance(color, position, scale, depth_bounds, fixed_scale);
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
    }

    pub(crate) fn render(&mut self, background_color: Color) {
        let mut rendering = Rendering::new(&self.renderer);
        let mut commands = RenderCommands::new(background_color.into(), &mut rendering);
        self.opaque_instances
            .render(&mut commands, &self.renderer, &self.shaders, &self.models);
        self.transparent_instances.render(
            &mut commands,
            &self.renderer,
            &self.shaders,
            &self.models,
        );
        drop(commands);
        rendering.apply();
    }

    fn fixed_scale(&self) -> (f32, f32) {
        let size = self.renderer.target_size();
        let width_scale = if size.0 > size.1 {
            size.1 as f32 / size.0 as f32
        } else {
            1.
        };
        let height_scale = if size.0 > size.1 {
            1.
        } else {
            size.0 as f32 / size.1 as f32
        };
        (width_scale, height_scale)
    }

    fn depth_bounds<'a, I>(depths: I) -> (f32, f32)
    where
        I: Iterator<Item = f32>,
    {
        let (min_z, mut max_z) = depths.fold((f32::INFINITY, 0_f32), |(min, max), b| {
            (min.min(b), max.max(b))
        });
        if min_z == max_z {
            max_z += f32::EPSILON; // avoid division by zero when creating instance
        }
        (min_z, max_z)
    }

    fn create_instance(
        color: &ShapeColor,
        position: &Position,
        scale: Option<&Scale>,
        depth_bounds: (f32, f32),
        fixed_scale: (f32, f32),
    ) -> Instance {
        let (min_z, max_z) = depth_bounds;
        let (x_scale, y_scale) = fixed_scale;
        let scale = scale.unwrap_or(&DEFAULT_SCALE).abs();
        let z_position = (1. - (position.abs().z - min_z) / (max_z - min_z)) * MAX_2D_DEPTH;
        Instance {
            transform: [
                [scale.x * 2. * x_scale, 0., 0., 0.],
                [0., scale.y * 2. * y_scale, 0., 0.],
                [0., 0., 0., 0.],
                [
                    position.abs().x * 2. * x_scale,
                    position.abs().y * 2. * y_scale,
                    z_position,
                    1.,
                ],
            ],
            color: [color.0.r, color.0.g, color.0.b, color.0.a],
        }
    }
}