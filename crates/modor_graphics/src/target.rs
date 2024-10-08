use crate::gpu::Gpu;
use crate::material::MaterialManager;
use crate::mesh::Mesh;
use crate::size::NonZeroSize;
use crate::{
    validation, AntiAliasingMode, Camera2DGlob, Color, InstanceGroup2DProperties, InstanceGroups2D,
    Mat, Shader, Size, Texture,
};
use log::{error, trace};
use modor::{App, FromApp, Global, Globals, StateHandle};
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
///         let target = app.get_mut::<Window>().target.to_ref().get_mut(app);
///         target.background_color = Color::RED;
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Target {
    /// Background color used for rendering.
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
    size: Size,
    texture_format: TextureFormat,
    loaded: Option<LoadedTarget>,
    is_error_logged: bool,
    is_incompatible_anti_aliasing_logged: bool,
    old_anti_aliasing: AntiAliasingMode,
    index: usize,
    cameras: StateHandle<Globals<Camera2DGlob>>,
    materials: StateHandle<Globals<Mat>>,
    meshes: StateHandle<Globals<Mesh>>,
}

impl FromApp for Target {
    fn from_app(app: &mut App) -> Self {
        Self {
            background_color: Color::BLACK,
            anti_aliasing: AntiAliasingMode::None,
            supported_anti_aliasing_modes: vec![AntiAliasingMode::None],
            size: Size::ZERO,
            texture_format: Texture::DEFAULT_FORMAT,
            loaded: None,
            is_error_logged: false,
            is_incompatible_anti_aliasing_logged: false,
            old_anti_aliasing: AntiAliasingMode::None,
            index: 0,
            cameras: app.handle(),
            materials: app.handle(),
            meshes: app.handle(),
        }
    }
}

impl Global for Target {
    fn init(&mut self, _app: &mut App, index: usize) {
        self.index = index;
    }
}

impl Target {
    /// Returns the size of the target in pixels.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Returns the sorted list of all supported [`AntiAliasingMode`].
    pub fn supported_anti_aliasing_modes(&self) -> &[AntiAliasingMode] {
        &self.supported_anti_aliasing_modes
    }

    pub(crate) fn disable(&mut self) {
        self.loaded = None;
    }

    pub(crate) fn enable(&mut self, gpu: &Gpu, size: NonZeroSize, format: TextureFormat) {
        let anti_aliasing = self.fixed_anti_aliasing();
        self.size = size.into();
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
        self.old_anti_aliasing = self.anti_aliasing;
    }

    pub(crate) fn render(&mut self, app: &mut App, gpu: &Gpu, view: TextureView) {
        app.take(MaterialManager::update_material_bind_groups);
        app.get_mut::<InstanceGroups2D>().sync(gpu);
        self.update_loaded(gpu);
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

    fn update_loaded(&mut self, gpu: &Gpu) {
        if self.anti_aliasing != self.old_anti_aliasing {
            let size = self.size.into();
            self.enable(gpu, size, self.texture_format);
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
                    camera
                        .targets
                        .iter()
                        .any(|target| target.index() == self.index)
                })
                && self
                    .materials
                    .get(app)
                    .get(group.material)
                    .map_or(false, |material| {
                        (material.is_transparent || material.has_transparent_texture)
                            == is_transparent
                    })
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
        let camera = self.cameras.get(app).get(group.camera)?;
        let mesh = self.meshes.get(app).get(group.mesh)?;
        let group = &groups.groups[&group];
        let primary_buffer = group.primary_buffer()?;
        let pipeline_params = (self.texture_format, anti_aliasing);
        pass.set_pipeline(shader.pipelines.get(&pipeline_params)?);
        pass.set_bind_group(Shader::CAMERA_GROUP, camera.bind_group(self.index)?, &[]);
        pass.set_bind_group(Shader::MATERIAL_GROUP, &material.bind_group.inner, &[]);
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
