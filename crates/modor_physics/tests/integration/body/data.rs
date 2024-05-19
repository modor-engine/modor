use modor::log::Level;
use modor::{App, Context, Globals, Node, RootNode, Visit};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::{Body2D, Body2DGlob};
use std::f32::consts::FRAC_PI_2;

#[modor::test]
fn retrieve_data() {
    let mut app = App::new::<Root>(Level::Info);
    let body_index = body(&mut app).glob().index();
    let data = &app.get_mut::<Globals<Body2DGlob>>()[body_index];
    assert_approx_eq!(data.position, Vec2::ZERO);
    assert_approx_eq!(data.size, Vec2::ONE);
    assert_approx_eq!(data.rotation, 0.);
    body(&mut app).position = Vec2::new(1., 2.);
    body(&mut app).size = Vec2::new(3., 4.);
    body(&mut app).rotation = FRAC_PI_2;
    app.update();
    let data = &app.get_mut::<Globals<Body2DGlob>>()[body_index];
    assert_approx_eq!(data.position, Vec2::new(1., 2.));
    assert_approx_eq!(data.size, Vec2::new(3., 4.));
    assert_approx_eq!(data.rotation, FRAC_PI_2);
}

fn body(app: &mut App) -> &mut Body2D {
    &mut app.get_mut::<Root>().body
}

#[derive(Node, Visit)]
struct Root {
    body: Body2D,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            body: Body2D::new(ctx),
        }
    }
}
