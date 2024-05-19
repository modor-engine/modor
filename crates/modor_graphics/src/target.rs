use crate::gpu::Gpu;
use crate::mesh::MeshGlob;
use crate::model::Instance;
use crate::size::NonZeroSize;
use crate::{
    validation, Camera2DGlob, Color, InstanceGroup2DKey, InstanceGroups2D, MaterialGlob,
    ShaderGlob, Size,
};
use log::{error, trace};
use modor::{Context, Glob, GlobRef, Globals, RootNodeHandle};
use std::any::TypeId;
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Extent3d, IndexFormat, LoadOp, Operations,
    RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    StoreOp, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

pub struct Target {
    /// Background color of the rendering.
    ///
    /// Default is [`Color::BLACK`].
    pub background_color: Color,
    loaded: Option<LoadedTarget>,
    is_error_logged: bool,
    label: String,
    glob: Glob<TargetGlob>,
    cameras: RootNodeHandle<Globals<Camera2DGlob>>,
    materials: RootNodeHandle<Globals<MaterialGlob>>,
    meshes: RootNodeHandle<Globals<MeshGlob>>,
}

impl Target {
    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<TargetGlob> {
        self.glob.as_ref()
    }

    pub(crate) fn new(ctx: &mut Context<'_>, label: String) -> Self {
        Self {
            background_color: Color::BLACK,
            loaded: None,
            is_error_logged: false,
            label,
            glob: Glob::new(ctx, TargetGlob { size: Size::ZERO }),
            cameras: ctx.root(),
            materials: ctx.root(),
            meshes: ctx.root(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.loaded = None;
    }

    pub(crate) fn init(
        &mut self,
        ctx: &mut Context<'_>,
        gpu: &Gpu,
        size: NonZeroSize,
        texture_format: TextureFormat,
    ) {
        self.glob.get_mut(ctx).size = size.into();
        self.loaded = Some(LoadedTarget {
            size,
            texture_format,
            color_buffer_view: Self::create_color_buffer_view(gpu, size, texture_format),
            depth_buffer_view: Self::create_depth_buffer_view(gpu, size),
        });
    }

    pub(crate) fn update(
        &mut self,
        ctx: &mut Context<'_>,
        gpu: &Gpu,
        size: NonZeroSize,
        texture_format: TextureFormat,
    ) {
        if let Some(loaded) = &mut self.loaded {
            if size != loaded.size {
                self.glob.get_mut(ctx).size = size.into();
                loaded.size = size;
                loaded.texture_format = texture_format;
                loaded.color_buffer_view =
                    Self::create_color_buffer_view(gpu, size, texture_format);
                loaded.depth_buffer_view = Self::create_depth_buffer_view(gpu, size);
            }
        }
    }

    pub(crate) fn render(&mut self, ctx: &mut Context<'_>, gpu: &Gpu, view: TextureView) {
        let loaded = self
            .loaded
            .as_ref()
            .expect("internal error: target not loaded");
        let mut encoder = Self::create_encoder(gpu);
        let mut pass = Self::create_pass(self.background_color, &mut encoder, &view, loaded);
        let groups = ctx.root::<InstanceGroups2D>().get(ctx);
        self.render_instance_groups(ctx, loaded, groups, &mut pass);
        let result = validation::validate_wgpu(gpu, || drop(pass));
        let is_err = result.is_err();
        if !is_err {
            gpu.queue.submit(Some(encoder.finish()));
        }
        trace!("Target '{}' rendered (error: {})", self.label, is_err);
        self.log_error(result);
    }

    fn render_instance_groups<'a>(
        &self,
        ctx: &'a Context<'_>,
        loaded: &LoadedTarget,
        groups: &'a InstanceGroups2D,
        pass: &mut RenderPass<'a>,
    ) {
        let mut sorted_groups: Vec<_> = groups.group_iter().collect();
        sorted_groups.sort_unstable();
        for group in sorted_groups {
            self.render_group(ctx, pass, group, groups, loaded);
        }
    }

    fn create_color_buffer_view(
        gpu: &Gpu,
        size: NonZeroSize,
        texture_format: TextureFormat,
    ) -> TextureView {
        let texture = gpu.device.create_texture(&TextureDescriptor {
            label: Some("modor_color_texture"),
            size: Extent3d {
                width: size.width.into(),
                height: size.height.into(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: texture_format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&TextureViewDescriptor::default())
    }

    fn create_depth_buffer_view(gpu: &Gpu, size: NonZeroSize) -> TextureView {
        let texture = gpu.device.create_texture(&TextureDescriptor {
            label: Some("modor_depth_texture"),
            size: Extent3d {
                width: size.width.into(),
                height: size.height.into(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&TextureViewDescriptor::default())
    }

    fn create_encoder(gpu: &Gpu) -> CommandEncoder {
        let descriptor = CommandEncoderDescriptor {
            label: Some("modor_render_encoder"),
        };
        gpu.device.create_command_encoder(&descriptor)
    }

    fn create_pass<'a>(
        background_color: Color,
        encoder: &'a mut CommandEncoder,
        view: &'a TextureView,
        loaded: &'a LoadedTarget,
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("modor_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(background_color.into()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &loaded.depth_buffer_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    // TODO: support transparency
    fn render_group<'a>(
        &self,
        ctx: &'a Context<'_>,
        pass: &mut RenderPass<'a>,
        group: InstanceGroup2DKey,
        groups: &'a InstanceGroups2D,
        loaded: &LoadedTarget,
    ) -> Option<()> {
        let material = self.materials.get(ctx).get(group.material)?;
        let shader = material.shader.get(ctx);
        if material.binding_ids.bind_group_layout != shader.material_bind_group_layout.global_id() {
            return None;
        }
        let camera = self.cameras.get(ctx).get(group.camera)?;
        let mesh = self.meshes.get(ctx).get(group.mesh)?;
        let group = &groups.groups[&group];
        let main_buffer = group.buffers[&TypeId::of::<Instance>()].buffer.as_ref()?;
        pass.set_pipeline(shader.pipelines.get(&loaded.texture_format)?);
        pass.set_bind_group(
            ShaderGlob::CAMERA_GROUP,
            camera.bind_group(self.glob())?,
            &[],
        );
        pass.set_bind_group(ShaderGlob::MATERIAL_GROUP, &material.bind_group.inner, &[]);
        pass.set_index_buffer(mesh.index_buffer.slice(), IndexFormat::Uint16);
        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice());
        pass.set_vertex_buffer(1, main_buffer.slice());
        if material.has_instance_data {
            // TODO: support secondary instances
        }
        pass.draw_indexed(0..(mesh.index_count as u32), 0, 0..(0 as u32));
        Some(())
    }

    fn log_error(&mut self, result: Result<(), wgpu::Error>) {
        if !self.is_error_logged {
            if let Err(error) = result {
                error!("Error during rendering in target '{}': {error}", self.label);
                self.is_error_logged = true;
            }
        }
    }
}

struct LoadedTarget {
    size: NonZeroSize,
    texture_format: TextureFormat,
    color_buffer_view: TextureView,
    depth_buffer_view: TextureView,
}

#[non_exhaustive]
pub struct TargetGlob {
    pub size: Size,
}
