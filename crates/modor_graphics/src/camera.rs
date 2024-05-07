use crate::buffer::{Buffer, BufferBindGroup};
use crate::gpu::{Gpu, GpuHandle, GpuManager, GpuState};
use crate::{Size, TargetGlob};
use fxhash::FxHashMap;
use modor::{Context, Glob, GlobRef, NoVisit, Node};
use modor_physics::modor_math::{Mat4, Quat, Vec2, Vec3};
use std::collections::hash_map::Entry;
use std::sync::Arc;
use wgpu::{BindGroup, BufferUsages};

#[derive(Debug, NoVisit)]
pub struct Camera2D {
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: f32,
    pub targets: Vec<GlobRef<TargetGlob>>,
    glob: Glob<Camera2DGlob>,
    label: String,
    gpu: GpuHandle,
}

impl Node for Camera2D {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        let target_sizes = self.target_sizes(ctx);
        let gpu = self.gpu.get(ctx);
        let glob = self.glob.get_mut(ctx);
        let Some(gpu) = self.retrieve_gpu(gpu, glob) else {
            return;
        };
        for (target, target_size) in self.targets.iter().zip(target_sizes) {
            let transform = self.gpu_transform(target_size.into());
            glob.update_target(&gpu, target.index(), transform, &self.label);
        }
    }
}

impl Camera2D {
    pub fn new(
        ctx: &mut Context<'_>,
        label: impl Into<String>,
        targets: Vec<GlobRef<TargetGlob>>,
    ) -> Self {
        let gpu_version = ctx.root::<GpuManager>().get(ctx).current_version;
        Self {
            position: Vec2::ZERO,
            size: Vec2::ONE,
            rotation: 0.,
            targets,
            glob: Glob::new(ctx, Camera2DGlob::new(gpu_version)),
            label: label.into(),
            gpu: GpuHandle::default(),
        }
    }

    // TODO: Option<Camera2DGlob> ?
    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<Camera2DGlob> {
        self.glob.as_ref()
    }

    /// Converts a `target_position` for a target surface of size `target_size` into world position.
    ///
    /// `target_position` with a value of [`Vec2::ZERO`] corresponds to top-left corner of the
    /// surface, and a value of [`Window::size()`](crate::Window::size) corresponds to the
    /// bottom-right corner.
    pub fn world_position(&self, target_size: Size, target_position: Vec2) -> Vec2 {
        let target_size = target_size.into();
        self.world_transform(target_size)
            * Vec2::new(
                target_position.x / target_size.x - 0.5,
                0.5 - target_position.y / target_size.y,
            )
    }

    fn gpu_transform(&self, target_size: Vec2) -> Mat4 {
        let x_scale = 1.0_f32.min(target_size.y / target_size.x);
        let y_scale = 1.0_f32.min(target_size.x / target_size.y);
        let position = Vec3::new(-self.position.x, -self.position.y, -1.);
        let scale = Vec3::new(2. * x_scale / self.size.x, 2. * y_scale / self.size.y, -1.);
        Mat4::from_position(position)
            * Quat::from_z(self.rotation).matrix()
            * Mat4::from_scale(scale)
    }

    fn world_transform(&self, target_size: Vec2) -> Mat4 {
        let x_scale = 1.0_f32.min(target_size.y / target_size.x);
        let y_scale = 1.0_f32.min(target_size.x / target_size.y);
        let scale = self.size.with_scale(Vec2::new(1. / x_scale, 1. / y_scale));
        Mat4::from_scale(scale.with_z(1.))
            * Quat::from_z(-self.rotation).matrix()
            * Mat4::from_position(self.position.with_z(0.))
    }

    fn target_sizes(&self, ctx: &Context<'_>) -> Vec<Size> {
        self.targets
            .iter()
            .map(|target| target.get(ctx).size)
            .collect()
    }

    fn retrieve_gpu(&mut self, gpu: GpuState, glob: &mut Camera2DGlob) -> Option<Arc<Gpu>> {
        match gpu {
            GpuState::None => {
                *glob = Camera2DGlob::new(0);
                None
            }
            GpuState::New(gpu) => {
                *glob = Camera2DGlob::new(gpu.version);
                Some(gpu)
            }
            GpuState::Same(gpu) => {
                glob.remove_old_targets(&self.targets);
                Some(gpu)
            }
        }
    }
}

#[derive(Debug)]
pub struct Camera2DGlob {
    target_uniforms: FxHashMap<usize, CameraUniform>,
    gpu_version: u64,
}

impl Camera2DGlob {
    pub(crate) fn bind_group(
        &self,
        target: &GlobRef<TargetGlob>,
        gpu_version: u64,
    ) -> Option<&BindGroup> {
        (self.gpu_version == gpu_version)
            .then(|| self.target_uniforms.get(&target.index()))
            .flatten()
            .map(|uniform| &uniform.bind_group.inner)
    }

    fn new(gpu_version: u64) -> Self {
        Self {
            target_uniforms: FxHashMap::default(),
            gpu_version,
        }
    }

    fn remove_old_targets(&mut self, targets: &[GlobRef<TargetGlob>]) {
        let target_indexes: Vec<_> = targets.iter().map(GlobRef::index).collect();
        self.target_uniforms
            .retain(|target_index, _| target_indexes.contains(target_index));
    }

    fn update_target(&mut self, gpu: &Gpu, target_index: usize, transform: Mat4, label: &str) {
        match self.target_uniforms.entry(target_index) {
            Entry::Occupied(mut entry) => entry.get_mut().update(gpu, transform),
            Entry::Vacant(entry) => {
                entry.insert(CameraUniform::new(gpu, transform, label));
            }
        }
    }
}

#[derive(Debug)]
struct CameraUniform {
    bind_group: BufferBindGroup,
    buffer: Buffer<[[f32; 4]; 4]>,
    transform: Mat4,
}

impl CameraUniform {
    const BINDING: u32 = 0;

    fn new(gpu: &Gpu, transform: Mat4, label: &str) -> Self {
        let label = format!("camera_2d:{label}");
        let buffer = Buffer::new(
            gpu,
            &[transform.to_array()],
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            &label,
        );
        Self {
            bind_group: BufferBindGroup::uniform(
                gpu,
                &buffer,
                Self::BINDING,
                &gpu.camera_bind_group_layout,
                &label,
            ),
            buffer,
            transform,
        }
    }

    fn update(&mut self, gpu: &Gpu, transform: Mat4) {
        if transform != self.transform {
            self.buffer.update(gpu, &[transform.to_array()]);
            self.transform = transform;
        }
    }
}
