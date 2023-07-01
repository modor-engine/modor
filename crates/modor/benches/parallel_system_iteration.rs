//! Tests performance of parallel execution of systems.

#[macro_use]
extern crate modor;

use criterion::{criterion_main, Criterion};
use modor::{App, BuiltEntity, EntityBuilder};

#[derive(Component, NoSystem)]
struct A(f32);

#[derive(Component, NoSystem)]
struct B(f32);

#[derive(Component, NoSystem)]
struct C(f32);

#[derive(Component, NoSystem)]
struct D(f32);

#[derive(Component, NoSystem)]
struct E(f32);

#[derive(Component)]
struct Item1;

#[systems]
impl Item1 {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new()
            .component(Self)
            .component(A(0.0))
            .component(B(0.0))
    }

    #[run]
    fn ab(a: &mut A, b: &mut B) {
        std::mem::swap(&mut a.0, &mut b.0);
    }

    #[run]
    fn cd(c: &mut C, d: &mut D) {
        std::mem::swap(&mut c.0, &mut d.0);
    }

    #[run]
    fn ce(c: &mut C, e: &mut E) {
        std::mem::swap(&mut c.0, &mut e.0);
    }
}

#[derive(Component)]
struct Item2;

#[systems]
impl Item2 {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new()
            .component(Self)
            .component(A(0.0))
            .component(B(0.0))
            .component(C(0.0))
    }

    #[run]
    fn ab(a: &mut A, b: &mut B) {
        std::mem::swap(&mut a.0, &mut b.0);
    }

    #[run]
    fn cd(c: &mut C, d: &mut D) {
        std::mem::swap(&mut c.0, &mut d.0);
    }

    #[run]
    fn ce(c: &mut C, e: &mut E) {
        std::mem::swap(&mut c.0, &mut e.0);
    }
}

#[derive(Component)]
struct Item3;

#[systems]
impl Item3 {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new()
            .component(Self)
            .component(A(0.0))
            .component(B(0.0))
            .component(C(0.0))
            .component(D(0.0))
    }

    #[run]
    fn ab(a: &mut A, b: &mut B) {
        std::mem::swap(&mut a.0, &mut b.0);
    }

    #[run]
    fn cd(c: &mut C, d: &mut D) {
        std::mem::swap(&mut c.0, &mut d.0);
    }

    #[run]
    fn ce(c: &mut C, e: &mut E) {
        std::mem::swap(&mut c.0, &mut e.0);
    }
}

#[derive(Component)]
struct Item4;

#[systems]
impl Item4 {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new()
            .component(Self)
            .component(A(0.0))
            .component(B(0.0))
            .component(C(0.0))
            .component(E(0.0))
    }

    #[run]
    fn ab(a: &mut A, b: &mut B) {
        std::mem::swap(&mut a.0, &mut b.0);
    }

    #[run]
    fn cd(c: &mut C, d: &mut D) {
        std::mem::swap(&mut c.0, &mut d.0);
    }

    #[run]
    fn ce(c: &mut C, e: &mut E) {
        std::mem::swap(&mut c.0, &mut e.0);
    }
}

fn run(c: &mut Criterion) {
    let mut app = App::new().with_thread_count(3);
    for _ in 0..10_000 {
        app = app.with_entity(Item1::build());
    }
    for _ in 0..10_000 {
        app = app.with_entity(Item2::build());
    }
    for _ in 0..10_000 {
        app = app.with_entity(Item3::build());
    }
    for _ in 0..10_000 {
        app = app.with_entity(Item4::build());
    }
    c.bench_function("parallel_system_iteration", |b| b.iter(|| app.update()));
}

mod group {
    criterion::criterion_group!(benches, super::run);
}
criterion_main!(group::benches);
