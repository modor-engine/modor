use crate::gpu::Gpu;
use crate::mesh::Mesh;
use crate::shader::glob::ShaderGlobInner;
use crate::size::NonZeroSize;
use crate::{
    validation, AntiAliasingMode, Camera2DGlob, Color, InstanceGroup2DProperties, InstanceGroups2D,
    MaterialGlob, Size, Texture,
};
use log::{error, trace};
use modor::{App, FromApp, Glob, Global, Globals, StateHandle};
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Extent3d, IndexFormat, LoadOp, Operations,
    RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    StoreOp, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

/// The target for a rendering.
///
/// The models can be rendered either in the [`Window`](crate::Window) target,
/// or in a created [`Texture`] target.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// #
/// #[derive(FromApp)]
/// struct Root;
///
/// impl State for Root {
///     fn init(&mut self, app: &mut App) {
///         app.get_mut::<Window>().target.background_color = Color::RED;
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Target {
    /// Background color of the rendering.
    ///
    /// Default is [`Color::BLACK`].
    pub background_color: Color,
    /// Anti-aliasing mode.
    ///
    /// If the mode is not supported, then no anti-aliasing is applied.
    ///
    /// Default is [`AntiAliasingMode::None`].
    pub anti_aliasing: AntiAliasingMode,
    pub(crate) supported_anti_aliasing_modes: Vec<AntiAliasingMode>,
    texture_format: TextureFormat,
    loaded: Option<LoadedTarget>,
    is_error_logged: bool,
    is_incompatible_anti_aliasing_logged: bool,
    glob: Glob<TargetGlob>,
    cameras: StateHandle<Globals<Camera2DGlob>>,
    materials: StateHandle<Globals<MaterialGlob>>,
    meshes: StateHandle<Globals<Mesh>>,
}

impl Target {
    /// Returns a reference to global data.
    pub fn glob(&self) -> &Glob<TargetGlob> {
        &self.glob
    }

    /// Returns the sorted list of all supported [`AntiAliasingMode`].
    pub fn supported_anti_aliasing_modes(&self) -> &[AntiAliasingMode] {
        &self.supported_anti_aliasing_modes
    }

    pub(crate) fn new(app: &mut App) -> Self {
        Self {
            background_color: Color::BLACK,
            anti_aliasing: AntiAliasingMode::None,
            supported_anti_aliasing_modes: vec![AntiAliasingMode::None],
            texture_format: Texture::DEFAULT_FORMAT,
            loaded: None,
            is_error_logged: false,
            is_incompatible_anti_aliasing_logged: false,
            glob: Glob::from_app(app),
            cameras: app.handle(),
            materials: app.handle(),
            meshes: app.handle(),
        }
    }

    pub(crate) fn disable(&mut self) {
        self.loaded = None;
    }

    pub(crate) fn enable(
        &mut self,
        app: &mut App,
        gpu: &Gpu,
        size: NonZeroSize,
        format: TextureFormat,
    ) {
        let glob = self.glob.get_mut(app);
        glob.size = size.into();
        glob.anti_aliasing = self.anti_aliasing;
        let anti_aliasing = self.fixed_anti_aliasing();
        self.texture_format = format;
        self.loaded = Some(LoadedTarget {
            color_buffer_view: Self::create_color_buffer_view(
                gpu,
                size,
                self.texture_format,
                anti_aliasing,
            ),
            depth_buffer_view: Self::create_depth_buffer_view(gpu, size, anti_aliasing),
        });
    }

    pub(crate) fn render(&mut self, app: &mut App, gpu: &Gpu, view: TextureView) {
        app.get_mut::<InstanceGroups2D>().sync(gpu);
        self.update_loaded(app, gpu);
        let anti_aliasing = self.fixed_anti_aliasing();
        let loaded = self
            .loaded
            .as_ref()
            .expect("internal error: target not loaded");
        let mut encoder = Self::create_encoder(gpu);
        let mut pass = Self::create_pass(
            self.background_color,
            anti_aliasing,
            &mut encoder,
            &view,
            loaded,
        );
        let groups = app.handle::<InstanceGroups2D>().get(app);
        self.render_opaque_groups(app, groups, &mut pass, anti_aliasing);
        self.render_transparent_groups(app, groups, &mut pass, anti_aliasing);
        let result = validation::validate_wgpu(gpu, false, || drop(pass));
        let is_err = result.is_err();
        if !is_err {
            gpu.queue.submit(Some(encoder.finish()));
        }
        trace!("Target rendered (error: {})", is_err);
        self.log_error(result);
    }

    fn update_loaded(&mut self, app: &mut App, gpu: &Gpu) {
        let glob = self.glob.get_mut(app);
        if self.anti_aliasing != glob.anti_aliasing {
            glob.anti_aliasing = self.anti_aliasing;
            let size = glob.size.into();
            self.enable(app, gpu, size, self.texture_format);
        }
    }

    fn create_color_buffer_view(
        gpu: &Gpu,
        size: NonZeroSize,
        texture_format: TextureFormat,
        anti_aliasing: AntiAliasingMode,
    ) -> TextureView {
        let texture = gpu.device.create_texture(&TextureDescriptor {
            label: Some("modor_color_texture"),
            size: Extent3d {
                width: size.width.into(),
                height: size.height.into(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: anti_aliasing.sample_count(),
            dimension: TextureDimension::D2,
            format: texture_format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&TextureViewDescriptor::default())
    }

    fn create_depth_buffer_view(
        gpu: &Gpu,
        size: NonZeroSize,
        anti_aliasing: AntiAliasingMode,
    ) -> TextureView {
        let texture = gpu.device.create_texture(&TextureDescriptor {
            label: Some("modor_depth_texture"),
            size: Extent3d {
                width: size.width.into(),
                height: size.height.into(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: anti_aliasing.sample_count(),
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
        anti_aliasing: AntiAliasingMode,
        encoder: &'a mut CommandEncoder,
        view: &'a TextureView,
        loaded: &'a LoadedTarget,
    ) -> RenderPass<'a> {
        let sample_count = anti_aliasing.sample_count();
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("modor_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: if sample_count > 1 {
                    &loaded.color_buffer_view
                } else {
                    view
                },
                resolve_target: (sample_count > 1).then_some(view),
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

    fn render_opaque_groups<'a>(
        &self,
        app: &'a App,
        groups: &'a InstanceGroups2D,
        pass: &mut RenderPass<'a>,
        anti_aliasing: AntiAliasingMode,
    ) {
        let mut sorted_groups: Vec<_> = self.group_iter(app, groups, false).collect();
        sorted_groups.sort_unstable();
        for group in sorted_groups {
            self.render_group(app, pass, group, None, groups, anti_aliasing);
        }
    }

    fn render_transparent_groups<'a>(
        &self,
        app: &'a App,
        groups: &'a InstanceGroups2D,
        pass: &mut RenderPass<'a>,
        anti_aliasing: AntiAliasingMode,
    ) {
        let mut sorted_instances: Vec<_> = self
            .group_iter(app, groups, true)
            .flat_map(|group| {
                groups.groups[&group]
                    .z_indexes
                    .iter()
                    .enumerate()
                    .map(move |(instance_index, z)| (group, instance_index, z))
            })
            .collect();
        sorted_instances.sort_unstable_by(|(group1, _, z1), (group2, _, z2)| {
            z1.total_cmp(z2).then(group1.cmp(group2))
        });
        for (group, instance_index, _) in sorted_instances {
            self.render_group(
                app,
                pass,
                group,
                Some(instance_index),
                groups,
                anti_aliasing,
            );
        }
    }

    fn group_iter<'a>(
        &'a self,
        app: &'a App,
        groups: &'a InstanceGroups2D,
        is_transparent: bool,
    ) -> impl Iterator<Item = InstanceGroup2DProperties> + 'a {
        groups.group_iter().filter(move |group| {
            self.cameras
                .get(app)
                .get(group.camera)
                .map_or(false, |camera| {
                    camera.targets.contains(&self.glob().to_ref())
                })
                && self
                    .materials
                    .get(app)
                    .get(group.material)
                    .map_or(false, |material| material.is_transparent == is_transparent)
        })
    }

    #[allow(clippy::cast_possible_truncation, clippy::range_plus_one)]
    fn render_group<'a>(
        &self,
        app: &'a App,
        pass: &mut RenderPass<'a>,
        group: InstanceGroup2DProperties,
        instance_index: Option<usize>,
        groups: &'a InstanceGroups2D,
        anti_aliasing: AntiAliasingMode,
    ) -> Option<()> {
        let material = self.materials.get(app).get(group.material)?;
        let shader = material.shader.get(app);
        if material.binding_ids.bind_group_layout
            != shader.glob.material_bind_group_layout.global_id()
        {
            return None;
        }
        let camera = self.cameras.get(app).get(group.camera)?;
        let mesh = self.meshes.get(app).get(group.mesh)?;
        let group = &groups.groups[&group];
        let primary_buffer = group.primary_buffer()?;
        let pipeline_params = (self.texture_format, anti_aliasing);
        pass.set_pipeline(shader.glob.pipelines.get(&pipeline_params)?);
        pass.set_bind_group(
            ShaderGlobInner::CAMERA_GROUP,
            camera.bind_group(self.glob())?,
            &[],
        );
        pass.set_bind_group(
            ShaderGlobInner::MATERIAL_GROUP,
            &material.bind_group.inner,
            &[],
        );
        pass.set_index_buffer(mesh.index_buffer.slice(), IndexFormat::Uint16);
        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice());
        pass.set_vertex_buffer(1, primary_buffer.slice());
        if let Some(buffer) = group.secondary_buffer() {
            pass.set_vertex_buffer(2, buffer.slice());
        }
        pass.draw_indexed(
            0..(mesh.index_buffer.len() as u32),
            0,
            if let Some(index) = instance_index {
                index as u32..index as u32 + 1
            } else {
                0..group.model_indexes.len() as u32
            },
        );
        Some(())
    }

    // coverage: off (difficult to test)
    fn log_error(&mut self, result: Result<(), wgpu::Error>) {
        if !self.is_error_logged {
            if let Err(error) = result {
                error!("Error during target rendering: {error}");
                self.is_error_logged = true;
            }
        }
    }
    // coverage: on

    fn fixed_anti_aliasing(&mut self) -> AntiAliasingMode {
        if self
            .supported_anti_aliasing_modes
            .contains(&self.anti_aliasing)
        {
            self.anti_aliasing
        } else {
            if !self.is_incompatible_anti_aliasing_logged {
                error!("Unsupported anti-aliasing mode: `{:?}`", self.anti_aliasing);
                self.is_incompatible_anti_aliasing_logged = true;
            }
            AntiAliasingMode::None
        }
    }
}

#[derive(Debug)]
struct LoadedTarget {
    #[allow(dead_code)] // will be used when supporting antialiasing
    color_buffer_view: TextureView,
    depth_buffer_view: TextureView,
}

/// The global data of a [`Target`].
#[non_exhaustive]
#[derive(Debug, Global)]
pub struct TargetGlob {
    /// Size of the target in pixels.
    pub size: Size,
    anti_aliasing: AntiAliasingMode,
}

impl Default for TargetGlob {
    fn default() -> Self {
        Self {
            size: Size::ZERO,
            anti_aliasing: AntiAliasingMode::None,
        }
    }
}
