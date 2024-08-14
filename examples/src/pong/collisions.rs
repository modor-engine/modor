use modor::{App, FromApp, Glob, State};
use modor_physics::{CollisionGroup, CollisionGroupUpdater, Impulse};

#[derive(FromApp)]
pub(crate) struct CollisionGroups {
    pub(crate) horizontal_wall: Glob<CollisionGroup>,
    pub(crate) vertical_wall: Glob<CollisionGroup>,
    pub(crate) paddle: Glob<CollisionGroup>,
    pub(crate) ball: Glob<CollisionGroup>,
}

impl State for CollisionGroups {
    fn init(&mut self, app: &mut App) {
        CollisionGroupUpdater::new(&self.paddle).add_impulse(
            app,
            &self.horizontal_wall,
            Impulse::new(0., 0.),
        );
        CollisionGroupUpdater::new(&self.ball)
            .add_impulse(app, &self.horizontal_wall, Impulse::new(1., 0.))
            .add_sensor(app, &self.vertical_wall)
            .add_sensor(app, &self.paddle);
    }
}
