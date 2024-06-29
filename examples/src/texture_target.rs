use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::{Camera2D, Color, Size, Sprite2D, Texture, TextureSource, Window};
use modor_physics::modor_math::Vec2;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Visit)]
struct Root {
    target_rectangle: Sprite2D,
    inner_rectangle: Sprite2D,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        ctx.get_mut::<Window>().target.background_color = Color::GRAY;
        Self {
            target_rectangle: Self::target_rectangle(ctx),
            inner_rectangle: Self::inner_rectangle(ctx),
        }
    }
}

impl Node for Root {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        self.inner_rectangle.model.rotation += 0.01;
    }
}

impl Root {
    fn target_rectangle(ctx: &mut Context<'_>) -> Sprite2D {
        let target_texture = ctx.get_mut::<TextureTarget>().texture.glob().clone();
        Sprite2D::new(ctx, "target-rectangle").with_material(|m| m.texture = target_texture)
    }

    fn inner_rectangle(ctx: &mut Context<'_>) -> Sprite2D {
        let target_camera = ctx.get_mut::<TextureTarget>().camera.glob().clone();
        Sprite2D::new(ctx, "target-rectangle")
            .with_model(|m| m.size = Vec2::ONE * 0.2)
            .with_model(|m| m.camera = target_camera)
            .with_material(|m| m.color = Color::RED)
    }
}

#[derive(Node, Visit)]
struct TextureTarget {
    texture: Res<Texture>,
    camera: Camera2D,
}

impl RootNode for TextureTarget {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let texture = Texture::new(ctx, "texture-target")
            .with_is_target_enabled(true)
            .with_target(|target| {
                target.anti_aliasing = target
                    .supported_anti_aliasing_modes()
                    .iter()
                    .copied()
                    .max()
                    .unwrap_or_default();
            })
            .load_from_source(ctx, TextureSource::Size(Size::new(300, 300)));
        let camera = Camera2D::new(ctx, "texture-target", vec![texture.target.glob().clone()]);
        Self { texture, camera }
    }
}
