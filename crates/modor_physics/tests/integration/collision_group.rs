use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_physics::{Body2D, Body2DUpdater, CollisionGroup, CollisionGroupUpdater};

#[modor::test]
fn drop_group() {
    let mut app = App::new::<Root>(Level::Info);
    let mut res = Resources::from_app_with(&mut app, Resources::init);
    app.update();
    assert_eq!(res.body2.get(&app).collisions().len(), 1);
    res.group2 = None;
    Body2DUpdater::default()
        .collision_group(None)
        .apply(&mut app, &res.body2);
    app.update();
    res.recreate_group2(&mut app);
    app.update();
    assert_eq!(res.body2.get(&app).collisions().len(), 0);
}

#[derive(FromApp, State)]
struct Root;

#[derive(FromApp)]
struct Resources {
    body1: Glob<Body2D>,
    body2: Glob<Body2D>,
    group1: Glob<CollisionGroup>,
    group2: Option<Glob<CollisionGroup>>,
}

impl Resources {
    fn init(&mut self, app: &mut App) {
        let group2 = Glob::from_app(app);
        CollisionGroupUpdater::new(&self.group1).add_sensor(app, &group2);
        Body2DUpdater::default()
            .collision_group(self.group1.to_ref())
            .apply(app, &self.body1);
        Body2DUpdater::default()
            .collision_group(group2.to_ref())
            .apply(app, &self.body2);
        self.group2 = Some(group2);
    }

    fn recreate_group2(&mut self, app: &mut App) {
        let group2 = Glob::from_app(app);
        Body2DUpdater::default()
            .collision_group(group2.to_ref())
            .apply(app, &self.body2);
        self.group2 = Some(group2);
    }
}
