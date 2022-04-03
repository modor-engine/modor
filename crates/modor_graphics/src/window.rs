use crate::backend::renderer::Renderer;
use crate::storages::core::CoreStorage;
use crate::{BackgroundColor, Color, ShapeColor};
use modor::{Built, EntityBuilder, Query, Single};
use modor_physics::{Position, Scale, Shape};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use winit::dpi::PhysicalSize;
use winit::window::Window as WinitWindow;

pub struct Window {
    window: Arc<RwLock<WinitWindow>>,
    core: CoreStorage,
}

#[singleton]
impl Window {
    pub(crate) fn build(window: Arc<RwLock<WinitWindow>>, renderer: Renderer) -> impl Built<Self> {
        EntityBuilder::new(Self {
            window,
            core: CoreStorage::new(renderer),
        })
    }

    pub fn size(&self) -> WindowSize {
        let size = self.read_winit_window().inner_size();
        WindowSize {
            width: size.width,
            height: size.height,
        }
    }

    pub fn set_size(&mut self, size: WindowSize) {
        let size = PhysicalSize::new(size.width, size.height);
        self.write_winit_window().set_inner_size(size);
    }

    pub fn set_title(&mut self, title: &str) {
        self.write_winit_window().set_title(title);
    }

    #[run]
    fn update_size(&mut self) {
        self.core.set_size(self.size());
    }

    #[run_after_previous]
    fn update(
        &mut self,
        shapes: Query<'_, (&ShapeColor, &Position, Option<&Scale>, Option<&Shape>)>,
    ) {
        self.core.update_instances(shapes);
    }

    #[run_after_previous]
    fn render(&mut self, background_color: Option<Single<'_, BackgroundColor>>) {
        let background_color = background_color.map_or(Color::BLACK, |c| c.0);
        self.core.render(background_color);
    }

    fn read_winit_window(&self) -> RwLockReadGuard<'_, WinitWindow> {
        self.window
            .read()
            .expect("internal error: cannot read inner window")
    }

    fn write_winit_window(&mut self) -> RwLockWriteGuard<'_, WinitWindow> {
        self.window
            .write()
            .expect("internal error: cannot write inner window")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
