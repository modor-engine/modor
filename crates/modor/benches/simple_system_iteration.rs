//! Tests performance of direct component iteration in systems.

#[macro_use]
extern crate modor;

use criterion::{criterion_main, Criterion};
use modor::{App, Built, EntityBuilder};

struct Vec3(f32, f32, f32);

struct Vec4(f32, f32, f32, f32);

struct Mat4(Vec4, Vec4, Vec4, Vec4);

struct Position(Vec3);

struct Rotation(Vec3);

struct Velocity(Vec3);

struct Object(Mat4);

#[entity]
impl Object {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(Mat4(
            Vec4(1., 0., 0., 0.),
            Vec4(0., 1., 0., 0.),
            Vec4(0., 0., 1., 0.),
            Vec4(0., 0., 0., 1.),
        )))
        .with(Position(Vec3(1., 0., 0.)))
        .with(Rotation(Vec3(1., 0., 0.)))
        .with(Velocity(Vec3(1., 0., 0.)))
    }

    #[run]
    fn update(velocity: &Velocity, position: &mut Position) {
        position.0 .0 += velocity.0 .0;
        position.0 .1 += velocity.0 .1;
        position.0 .2 += velocity.0 .2;
    }
}

fn run(c: &mut Criterion) {
    let mut app = App::new();
    for _ in 0..10_000 {
        app = app.with_entity(Object::build());
    }
    c.bench_function("simple_system_iteration", |b| b.iter(|| app.update()));
}

mod group {
    criterion::criterion_group!(benches, super::run);
}
criterion_main!(group::benches);
