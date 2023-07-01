//! Tests performance of entity creation.
#![allow(unused_tuple_struct_fields)]

#[macro_use]
extern crate modor;

use criterion::{criterion_main, Criterion};
use modor::{App, BuiltEntity, EntityBuilder};

struct Vec3(f32, f32, f32);

struct Vec4(f32, f32, f32, f32);

struct Mat4(Vec4, Vec4, Vec4, Vec4);

#[derive(Component, NoSystem)]
struct Position(Vec3);

#[derive(Component, NoSystem)]
struct Rotation(Vec3);

#[derive(Component, NoSystem)]
struct Velocity(Vec3);

#[derive(Component, NoSystem)]
struct Transform(Mat4);

fn build_entity() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform(Mat4(
            Vec4(1., 0., 0., 0.),
            Vec4(0., 1., 0., 0.),
            Vec4(0., 0., 1., 0.),
            Vec4(0., 0., 0., 1.),
        )))
        .component(Position(Vec3(1., 0., 0.)))
        .component(Rotation(Vec3(1., 0., 0.)))
        .component(Velocity(Vec3(1., 0., 0.)))
}

fn run(c: &mut Criterion) {
    c.bench_function("entity_creation", |b| {
        b.iter(|| {
            let mut app = App::new();
            for _ in 0..10_000 {
                app = app.with_entity(build_entity());
            }
        });
    });
}

mod group {
    criterion::criterion_group!(benches, super::run);
}
criterion_main!(group::benches);
