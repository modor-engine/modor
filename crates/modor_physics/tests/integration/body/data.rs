use modor::log::Level;
use modor::{App, FromApp, Globals, State};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::{Body2D, Body2DGlob};
use std::f32::consts::FRAC_PI_2;

#[modor::test]
fn retrieve_data() {
    let mut app = App::new::<Root>(Level::Info);
    let body_glob = body(&mut app).glob().to_ref();
    let data = &app.get_mut::<Globals<Body2DGlob>>()[&body_glob];
    assert_approx_eq!(data.position, Vec2::ZERO);
    assert_approx_eq!(data.size, Vec2::ONE);
    assert_approx_eq!(data.rotation, 0.);
    body(&mut app).position = Vec2::new(1., 2.);
    body(&mut app).size = Vec2::new(3., 4.);
    body(&mut app).rotation = FRAC_PI_2;
    app.update();
    let data = &app.get_mut::<Globals<Body2DGlob>>()[&body_glob];
    assert_approx_eq!(data.position, Vec2::new(1., 2.));
    assert_approx_eq!(data.size, Vec2::new(3., 4.));
    assert_approx_eq!(data.rotation, FRAC_PI_2);
}

fn body(app: &mut App) -> &mut Body2D {
    &mut app.get_mut::<Root>().body
}

struct Root {
    body: Body2D,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        Self {
            body: Body2D::new(app),
        }
    }
}

impl State for Root {
    fn update(&mut self, app: &mut App) {
        self.body.update(app);
    }
}
