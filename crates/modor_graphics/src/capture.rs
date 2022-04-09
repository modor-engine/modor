use crate::backend::renderer::Renderer;
use crate::capture::internal::{PrepareCaptureRenderingAction, RenderCaptureAction};
use crate::storages::core::CoreStorage;
use crate::{BackgroundColor, Color, ShapeColor, SurfaceSize};
use modor::{Built, EntityBuilder, Query, Single};
use modor_physics::{Position, Scale, Shape};
use std::io::{BufWriter, Write};

// TODO: create common entity for Capture and Window

pub struct Capture {
    core: CoreStorage,
    buffer: Vec<u8>,
}

#[singleton]
impl Capture {
    pub fn build(size: SurfaceSize) -> impl Built<Self> {
        EntityBuilder::new(Self {
            core: CoreStorage::new(Renderer::for_texture((size.width, size.height))),
            buffer: vec![],
        })
    }

    pub fn size(&self) -> SurfaceSize {
        let size = self.core.renderer().target_size();
        SurfaceSize::new(size.0, size.1)
    }

    pub fn set_size(&mut self, size: SurfaceSize) {
        self.core.set_size(size);
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    #[run_as(PrepareCaptureRenderingAction)]
    fn prepare_rendering(
        &mut self,
        shapes: Query<'_, (&ShapeColor, &Position, Option<&Scale>, Option<&Shape>)>,
    ) {
        self.core.update_instances(shapes);
    }

    #[run_as(RenderCaptureAction)]
    fn render(&mut self, background_color: Option<Single<'_, BackgroundColor>>) {
        let background_color = background_color.map_or(Color::BLACK, |c| c.color());
        self.core.render(background_color);
    }

    #[run_as(UpdateCaptureBuffer)]
    pub fn update_buffer(&mut self) {
        let target = self.core.target_view();
        let mut writer = BufWriter::new(Vec::new());
        target.use_buffer_slice(|s| {
            for chunk in s.chunks(target.padded_bytes_per_row()) {
                writer
                    .write_all(&chunk[..target.unpadded_bytes_per_row()])
                    .expect("internal error: cannot write capture buffer");
            }
        });
        self.buffer = writer
            .into_inner()
            .expect("internal error: cannot extract capture buffer");
    }
}

#[action(RenderCaptureAction)]
pub struct UpdateCaptureBuffer;

pub(crate) mod internal {
    use modor_physics::UpdatePhysicsAction;

    #[action(UpdatePhysicsAction)]
    pub struct PrepareCaptureRenderingAction;

    #[action(PrepareCaptureRenderingAction)]
    pub struct RenderCaptureAction;
}
