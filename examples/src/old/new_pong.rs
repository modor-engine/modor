use std::path::Component::ParentDir;

// TODO: how to deal with Sprite2D that needs Model2D and Material2D (same for Text2D)

fn update(app: &mut App) {
    match app.get_mut::<Root>(()) {
        Root::Menu => (),
        Root::Game => update_game(app),
    }
}

fn update_game(app: &mut App) {
    collision_groups::update(app);
    wall::update(app);
    paddle::update(app, PaddleSide::Left);
    paddle::update(app, PaddleSide::Right);
}

fn exit_game(app: &mut App) {
    *app.get_mut::<Root>(()) = State::Menu;
    app.delete_all(paddle::SCOPE);
    app.delete_all(collision_groups::SCOPE);
}

fn start_game(app: &mut App) {
    *app.get_mut::<Root>(()) = State::Game;
}

#[derive(Default, SingletonData)]
enum State {
    #[default]
    Menu,
    Game,
}

mod wall {
    pub(crate) fn update(app: &mut App) {
        const SCOPE: Scope = Scope::new("wall");
        app.get_mut::<Sprite2D>(SCOPE.key(0))
            .position(Vec2::new(-0.45, 0.))
            .size(Vec2::new(0.02, 0.9))
            .color(Color::WHITE);
    }
}

mod paddle {
    pub(crate) const SCOPE: Scope = Scope::new("paddle");
    const SIZE: Vec2 = Vec2::new(0.02, 0.9);
    const SPEED: f32 = 0.9;

    pub(crate) fn update(app: &mut App, side: PaddleSide) {
        init(app, side);
        let body = app.get_mut::<Body2D>(key(side)).set_velocity(velocity(app));
        let position = body.position();
        app.get_mut::<Sprite2D>(key(side))
            .set_position(position)
            .set_size(Vec2::new(0.02, 0.9))
            .set_color(Color::WHITE);
    }

    fn init(app: &mut App, side: PaddleSide) {
        if let Some(body) = app.init::<Body2D>(key(side)) {
            body.set_position(initial_position(side))
                .set_size(SIZE)
                .set_mass(1.)
                .set_group(Some(super::collision_groups::PADDLE));
        }
    }

    fn key(side: PaddleSide) -> Key<usize> {
        SCOPE.key(side as usize)
    }

    fn initial_position(side: PaddleSide) -> Vec2 {
        match side {
            PaddleSide::Left => -Vec2::X * 0.7,
            PaddleSide::Right => Vec2::X * 0.7,
        }
    }

    fn velocity(app: &mut App) -> Vec2 {
        app.get::<Inputs>()
            .keyboard
            .axis(Key::ArrowUp, Key::ArrowDown)
            * SPEED;
    }

    pub(crate) enum PaddleSide {
        Left,
        Right,
    }
}

mod collision_groups {
    pub(crate) const SCOPE: Scope = Scope::new("collision_groups");
    pub(crate) const PADDLE: Key<usize> = SCOPE.key(0);
    pub(crate) const BALL: Key<usize> = SCOPE.key(1);
    pub(crate) const WALL: Key<usize> = SCOPE.key(2);

    pub(crate) fn update(app: &mut App) {
        app.get_mut::<CollisionGroups>()
            .sensor(PADDLE, BALL)
            .impulse(PADDLE, BALL, Impulse::new(0., 1.))
    }
}

impl Data for Body2D {
    type Storage = PhysicsStorage; // VecStorage<T> | SingletonStorage<T> | PhysicsStorage | ...
}

impl PhysicsStorage {
    type Key = Key<usize>;
    type Access<'a> = Body2DAccess<'a>;
    type AccessMut<'a> = Body2DAccessMut<'a>;

    fn on_update(app: &mut App) -> Self {
        todo!()
    }

    fn get(app: &App, key: <Self::Storage as Storage>::Key) -> Self::Access<'_> {
        app.storage::<Self>()
        // ...
    }

    fn get_mut(app: &mut App, key: <Self::Storage as Storage>::Key) -> Self::AccessMut<'_> {
        app.storage_mut::<Self>()
        // ...
    }

    fn iter(
        app: &App,
        key: <Self::Storage as Storage>::Key,
    ) -> impl Iterator<Item = Self::Access<'_>> {
        todo!()
    }
}
