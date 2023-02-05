use crate::instances::opaque::{OpaqueInstanceManager, OpaqueInstances};
use crate::registries::cameras::{Camera2DRegistry, CameraData};
use crate::registries::models::ModelRegistry;
use crate::registries::shaders::ShaderRegistry;
use crate::resources::buffers::DynamicBuffer;
use crate::resources::models::Model;
use crate::resources::shaders::Shader;
use crate::resources::uniforms::Uniform;
use crate::targets::texture::TextureTarget;
use crate::targets::window::WindowTarget;
use crate::targets::{GpuDevice, Target};
use bytemuck::Pod;
use modor::{Built, EntityBuilder, Query, Single, SingleMut};
use std::ops::Range;
use wgpu::{BindGroupLayout, Device, IndexFormat, RenderPass, TextureFormat};

pub(crate) struct Rendering;

#[singleton]
impl Rendering {
    pub(crate) fn build(
        target_format: TextureFormat,
        device: &Device,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with_child(OpaqueInstanceManager::build())
            .with_child(ShaderRegistry::build())
            .with_child(ModelRegistry::build())
            .with_child(Camera2DRegistry::build())
            .with_child(Shader::build_rectangle(
                target_format,
                camera_bind_group_layout,
                device,
            ))
            .with_child(Shader::build_ellipse(
                target_format,
                camera_bind_group_layout,
                device,
            ))
            .with_child(Model::build_rectangle(device))
    }

    #[run_after(
        component(WindowTarget),
        component(TextureTarget),
        component(Camera2DRegistry),
        component(ShaderRegistry),
        component(ModelRegistry),
        component(Shader),
        component(Model),
        component(OpaqueInstances)
    )]
    fn prepare(
        mut target: SingleMut<'_, Target>,
        camera_registry: Single<'_, Camera2DRegistry>,
        shader_registry: Single<'_, ShaderRegistry>,
        model_registry: Single<'_, ModelRegistry>,
        shaders: Query<'_, &Shader>,
        models: Query<'_, &Model>,
        opaque_instances: Query<'_, &OpaqueInstances>,
    ) {
        let mut pass = target.begin_render_pass();
        for instances in opaque_instances.iter() {
            let resource_keys = instances.resource_keys();
            let shader_key = &resource_keys.shader;
            let model_key = &resource_keys.model;
            let camera_uniform = camera_registry.uniform(&resource_keys.camera);
            match (
                shader_registry.find(shader_key, &shaders),
                model_registry.find(model_key, &models),
            ) {
                (None, _) => panic!("internal error: not found shader '{:?}' ", shader_key),
                (_, None) => panic!("internal error: not found model '{:?}'", model_key),
                (Some(shader), Some(model)) => {
                    Self::draw(
                        &mut pass,
                        shader,
                        camera_uniform,
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
        shader: &'a Shader,
        camera_uniform: &'a Uniform<CameraData>,
        vertex_buffer: &'a DynamicBuffer<V>,
        index_buffer: &'a DynamicBuffer<u16>,
        instance_buffer: &'a DynamicBuffer<I>,
        drawn_instance_idxs: Range<usize>,
    ) where
        V: Pod + Sync + Send,
        I: Pod + Sync + Send,
    {
        pass.set_pipeline(shader.pipeline());
        pass.set_bind_group(Shader::CAMERA_GROUP, camera_uniform.bind_group(), &[]);
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
