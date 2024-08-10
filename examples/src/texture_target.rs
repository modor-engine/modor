use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_graphics::modor_resources::Res;
use modor_graphics::{Camera2D, Color, Size, Sprite2D, Texture, TextureSource, Window};
use modor_physics::modor_math::Vec2;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    target_rectangle: Sprite2D,
    inner_rectangle: Sprite2D,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        app.get_mut::<Window>().target.background_color = Color::GRAY;
        Self {
            target_rectangle: Self::target_rectangle(app),
            inner_rectangle: Self::inner_rectangle(app),
        }
    }
}

impl State for Root {
    fn update(&mut self, app: &mut App) {
        self.inner_rectangle.model.rotation += 0.01;
        self.target_rectangle.update(app);
        self.inner_rectangle.update(app);
    }
}

impl Root {
    fn target_rectangle(app: &mut App) -> Sprite2D {
        let target_texture = app.get_mut::<TextureTarget>().texture.to_ref();
        Sprite2D::new(app).with_material(|m| m.texture = target_texture)
    }

    fn inner_rectangle(app: &mut App) -> Sprite2D {
        let target_camera = app.get_mut::<TextureTarget>().camera.glob().to_ref();
        Sprite2D::new(app)
            .with_model(|m| m.size = Vec2::ONE * 0.2)
            .with_model(|m| m.camera = target_camera)
            .with_material(|m| m.color = Color::RED)
    }
}

struct TextureTarget {
    texture: Glob<Res<Texture>>,
    camera: Camera2D,
}

impl FromApp for TextureTarget {
    fn from_app(app: &mut App) -> Self {
        let texture = Glob::<Res<Texture>>::from_app(app);
        let camera = Camera2D::new(app, vec![texture.get(app).target.glob().to_ref()]);
        Self { texture, camera }
    }
}

impl State for TextureTarget {
    fn init(&mut self, app: &mut App) {
        let anti_aliasing = self
            .texture
            .get(app)
            .target
            .supported_anti_aliasing_modes()
            .iter()
            .copied()
            .max()
            .unwrap_or_default();
        self.texture
            .updater()
            .source(TextureSource::Size(Size::new(300, 300)))
            .inner(|i, _| i.is_target_enabled(true))
            .inner(|i, _| i.target_anti_aliasing(anti_aliasing))
            .apply(app);
    }

    fn update(&mut self, app: &mut App) {
        self.camera.update(app);
    }
}
