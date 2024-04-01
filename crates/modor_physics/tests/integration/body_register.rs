use modor::log::Level;
use modor::{App, Context, Node, RootNode, Visit};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::{Body2D, Body2DRegister};
use std::f32::consts::FRAC_PI_2;

#[modor::test]
fn retrieve_transform() {
    let mut app = App::new::<Root>(Level::Info);
    let body_index = body(&mut app).index();
    let transform = app.root::<Body2DRegister>().transform(&body_index);
    assert_approx_eq!(transform.position, Vec2::ZERO);
    assert_approx_eq!(transform.size, Vec2::ONE);
    assert_approx_eq!(transform.rotation, 0.);
    body(&mut app).position = Vec2::new(1., 2.);
    body(&mut app).size = Vec2::new(3., 4.);
    body(&mut app).rotation = FRAC_PI_2;
    app.update();
    let transform = app.root::<Body2DRegister>().transform(&body_index);
    assert_approx_eq!(transform.position, Vec2::new(1., 2.));
    assert_approx_eq!(transform.size, Vec2::new(3., 4.));
    assert_approx_eq!(transform.rotation, FRAC_PI_2);
}

fn body(app: &mut App) -> &mut Body2D {
    &mut app.root::<Root>().body
}

#[derive(Node, Visit)]
struct Root {
    body: Body2D,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            body: Body2D::new(ctx, Vec2::ZERO, Vec2::ONE),
        }
    }
}
