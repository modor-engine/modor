//! Tests performance of entity creation.

use criterion::{criterion_main, Criterion};
use modor::{App, Built, EntityBuilder, EntityMainComponent};

struct Vec3(f32, f32, f32);

struct Vec4(f32, f32, f32, f32);

struct Mat4(Vec4, Vec4, Vec4, Vec4);

struct Position(Vec3);

struct Rotation(Vec3);

struct Velocity(Vec3);

struct Object(Mat4);

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
}

impl EntityMainComponent for Object {
    type Type = ();
}

fn run(c: &mut Criterion) {
    c.bench_function("entity_creation", |b| {
        b.iter(|| {
            let mut app = App::new();
            for _ in 0..10_000 {
                app = app.with_entity(Object::build());
            }
        });
    });
}

mod group {
    criterion::criterion_group!(benches, super::run);
}
criterion_main!(group::benches);
