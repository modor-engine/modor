use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::{
    Camera2D, Color, DefaultMaterial2D, IntoMat, Mat, Model2D, Size, Texture, TextureSource, Window,
};
use modor_physics::modor_math::Vec2;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Node, Visit)]
struct Root {
    target_rectangle: TargetRectangle,
    inner_rectangle: InnerRectangle,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        ctx.get_mut::<Window>().target.background_color = Color::GRAY;
        Self {
            target_rectangle: TargetRectangle::new(ctx),
            inner_rectangle: InnerRectangle::new(ctx),
        }
    }
}

#[derive(Node, Visit)]
struct TargetRectangle {
    material: Mat<DefaultMaterial2D>,
    model: Model2D<DefaultMaterial2D>,
}

impl TargetRectangle {
    fn new(ctx: &mut Context<'_>) -> Self {
        let material = DefaultMaterial2D::new(ctx)
            .with_texture(ctx.get_mut::<TextureTarget>().texture.glob().clone())
            .into_mat(ctx, "inner-rectangle");
        let model = Model2D::new(ctx, material.glob());
        Self { material, model }
    }
}

#[derive(Visit)]
struct InnerRectangle {
    material: Mat<DefaultMaterial2D>,
    model: Model2D<DefaultMaterial2D>,
}

impl InnerRectangle {
    fn new(ctx: &mut Context<'_>) -> Self {
        let material = DefaultMaterial2D::new(ctx)
            .with_color(Color::RED)
            .into_mat(ctx, "inner-rectangle");
        let model = Model2D::new(ctx, material.glob())
            .with_size(Vec2::ONE * 0.2)
            .with_camera(ctx.get_mut::<TextureTarget>().camera.glob().clone());
        Self { material, model }
    }
}

impl Node for InnerRectangle {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        self.model.rotation += 0.01;
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
            .load_from_source(TextureSource::Size(Size::new(300, 300)));
        let camera = Camera2D::new(ctx, "texture-target", vec![texture.target.glob().clone()]);
        Self { texture, camera }
    }
}
