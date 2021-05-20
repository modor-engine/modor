use modor::*;

fn main() {
    Application::new().with_group(build_main_group).update();
}

fn build_main_group(builder: &mut GroupBuilder<'_>) {
    for i in 0..20 {
        if i % 10 == 0 {
            builder.with_entity::<DynamicBody>(i);
        } else {
            builder.with_entity::<StaticBody>(i);
        }
    }
}

#[derive(Debug)]
struct Position(f32, f32);

#[derive(Debug)]
struct Velocity(f32, f32);

struct StaticBody;

impl EntityMainComponent for StaticBody {
    type Params = usize;

    fn build(builder: &mut EntityBuilder<'_, Self>, value: Self::Params) -> Built {
        builder
            .with(Position(value as f32, value as f32 + 0.5))
            .with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(system!(Self::print));
        runner.run(system!(Self::count_velocity));
        runner.run(system!(Self::count_velocity_mut));
        runner.run(system!(Self::option));
    }
}

impl StaticBody {
    fn print(&self, position: &Position, query: Query<'_, (&Velocity,)>) {
        let mut n = 0;
        for_each!(query, |_vel: &Velocity| n += 1);
        println!("StaticBody({:?}), {}", position, n);
    }

    fn count_velocity(query: Query<'_, (&Velocity,)>) {
        let mut n = 0;
        for_each!(query, |_vel: &Velocity| n += 1);
        println!("Number of entities with velocity: {}", n);
    }

    fn count_velocity_mut(mut query: QueryMut<'_, (&mut Velocity,)>) {
        let mut n = 0;
        for_each_mut!(query, |_vel: &mut Velocity| n += 1);
        println!("Number of entities with velocity mut: {}", n);
    }

    fn option(position: &Position, velocity: Option<&Velocity>) {
        println!("=> ({:?}, {:?})", position, velocity);
    }
}

struct DynamicBody;

impl EntityMainComponent for DynamicBody {
    type Params = usize;

    fn build(builder: &mut EntityBuilder<'_, Self>, value: Self::Params) -> Built {
        builder
            .inherit_from::<StaticBody>(value)
            .with(Velocity(value as f32 + 0.25, value as f32 + 0.75))
            .with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(system!(Self::print));
    }
}

impl DynamicBody {
    fn print(&self, position: &Position, velocity: &Velocity) {
        println!("DynamicBody({:?}, {:?})", position, velocity);
    }
}
