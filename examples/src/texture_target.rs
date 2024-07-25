use modor::log::Level;
use modor::{App, Node, RootNode};
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::{Camera2D, Color, Size, Sprite2D, Texture, TextureSource, Window};
use modor_physics::modor_math::Vec2;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    target_rectangle: Sprite2D,
    inner_rectangle: Sprite2D,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        app.get_mut::<Window>().target.background_color = Color::GRAY;
        Self {
            target_rectangle: Self::target_rectangle(app),
            inner_rectangle: Self::inner_rectangle(app),
        }
    }
}

impl Node for Root {
    fn update(&mut self, app: &mut App) {
        self.inner_rectangle.model.rotation += 0.01;
        self.target_rectangle.update(app);
        self.inner_rectangle.update(app);
    }
}

impl Root {
    fn target_rectangle(app: &mut App) -> Sprite2D {
        let target_texture = app.get_mut::<TextureTarget>().texture.glob().to_ref();
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
    texture: Res<Texture>,
    camera: Camera2D,
}

impl RootNode for TextureTarget {
    fn on_create(app: &mut App) -> Self {
        let texture = Texture::new(app)
            .with_is_target_enabled(true)
            .with_target(|target| {
                target.anti_aliasing = target
                    .supported_anti_aliasing_modes()
                    .iter()
                    .copied()
                    .max()
                    .unwrap_or_default();
            })
            .load_from_source(app, TextureSource::Size(Size::new(300, 300)));
        let camera = Camera2D::new(app, vec![texture.target.glob().to_ref()]);
        Self { texture, camera }
    }
}

impl Node for TextureTarget {
    fn update(&mut self, app: &mut App) {
        self.texture.update(app);
        self.camera.update(app);
    }
}
