use crate::instances::opaque::{OpaqueInstanceManager, OpaqueInstances};
use crate::resources::buffers::DynamicBuffer;
use crate::resources::cameras::CameraData;
use crate::resources::models::{Model, ModelRegistry};
use crate::resources::shaders::{Shader, ShaderRegistry};
use crate::resources::uniforms::Uniform;
use crate::targets::texture::TextureTarget;
use crate::targets::window::WindowTarget;
use crate::targets::{GpuDevice, Target};
use crate::Camera2D;
use bytemuck::Pod;
use modor::{Built, EntityBuilder, Query, Single, SingleMut};
use std::ops::Range;
use wgpu::{Device, IndexFormat, RenderPass, TextureFormat};

const CAMERA_BINDING: u32 = 0;

pub(crate) struct Rendering;

#[singleton]
impl Rendering {
    pub(crate) fn build(target_format: TextureFormat, device: &Device) -> impl Built<Self> {
        let camera_2d_uniform =
            Uniform::new(CameraData::default(), CAMERA_BINDING, "camera_2d", device);
        EntityBuilder::new(Self)
            .with_child(OpaqueInstanceManager::build())
            .with_child(ShaderRegistry::build())
            .with_child(ModelRegistry::build())
            .with_child(Shader::build_rectangle(
                target_format,
                &camera_2d_uniform,
                device,
            ))
            .with_child(Shader::build_ellipse(
                target_format,
                &camera_2d_uniform,
                device,
            ))
            .with_child(Model::build_rectangle(device))
            .with_child(Camera2D::build(camera_2d_uniform))
    }

    #[run_after(
        entity(WindowTarget),
        entity(TextureTarget),
        entity(Camera2D),
        entity(ShaderRegistry),
        entity(ModelRegistry),
        entity(Shader),
        entity(Model),
        entity(OpaqueInstances)
    )]
    fn prepare(
        mut target: SingleMut<'_, Target>,
        camera: Single<'_, Camera2D>,
        shader_registry: Single<'_, ShaderRegistry>,
        model_registry: Single<'_, ModelRegistry>,
        shaders: Query<'_, &Shader>,
        models: Query<'_, &Model>,
        opaque_instances: Query<'_, &OpaqueInstances>,
    ) {
        let mut pass = target.begin_render_pass();
        camera.use_for_rendering(&mut pass);
        for instances in opaque_instances.iter() {
            let shader_key = &instances.resource_keys().shader;
            let model_key = &instances.resource_keys().model;
            match (
                shader_registry.find(shader_key, &shaders),
                model_registry.find(model_key, &models),
            ) {
                (None, _) => panic!("internal error: not found shader '{:?}' ", shader_key),
                (_, None) => panic!("internal error: not found model '{:?}'", model_key),
                (Some(shader), Some(model)) => {
                    shader.use_for_rendering(&mut pass);
                    Self::draw(
                        &mut pass,
                        model.vertex_buffer(),
                        model.index_buffer(),
                        instances.buffer(),
                        0..instances.buffer().len(),
                    );
                }
            }
        }
    }

    #[run_after_previous]
    fn copy_texture_target_to_buffer(
        texture_target: Single<'_, TextureTarget>,
        mut target: SingleMut<'_, Target>,
    ) {
        texture_target.copy_texture_to_buffer(&mut target);
    }

    #[run_after_previous]
    fn submit_command_queue(mut target: SingleMut<'_, Target>, device: Single<'_, GpuDevice>) {
        target.submit_command_queue(device);
    }

    #[run_after_previous]
    fn present_texture_to_window(mut window_target: SingleMut<'_, WindowTarget>) {
        window_target.present_texture();
    }

    #[allow(clippy::cast_possible_truncation)]
    fn draw<'a, V, I>(
        pass: &mut RenderPass<'a>,
        vertex_buffer: &'a DynamicBuffer<V>,
        index_buffer: &'a DynamicBuffer<u16>,
        instance_buffer: &'a DynamicBuffer<I>,
        drawn_instance_idxs: Range<usize>,
    ) where
        V: Pod + Sync + Send,
        I: Pod + Sync + Send,
    {
        pass.set_vertex_buffer(0, vertex_buffer.buffer());
        pass.set_vertex_buffer(1, instance_buffer.buffer());
        pass.set_index_buffer(index_buffer.buffer(), IndexFormat::Uint16);
        pass.draw_indexed(
            0..(index_buffer.len() as u32),
            0,
            (drawn_instance_idxs.start as u32)..(drawn_instance_idxs.end as u32),
        );
    }
}
