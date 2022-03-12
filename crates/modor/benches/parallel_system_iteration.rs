//! Tests performance of parallel execution of systems.

use criterion::{criterion_main, Criterion};
use modor::{system, App, Built, EntityBuilder, EntityMainComponent, SystemRunner};

struct A(f32);
struct B(f32);
struct C(f32);
struct D(f32);
struct E(f32);

struct Item1;

impl Item1 {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).with(A(0.0)).with(B(0.0))
    }
}

impl EntityMainComponent for Item1 {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(ab)).run(system!(cd)).run(system!(ce))
    }
}

struct Item2;

impl Item2 {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(A(0.0))
            .with(B(0.0))
            .with(C(0.0))
    }
}

impl EntityMainComponent for Item2 {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(ab)).run(system!(cd)).run(system!(ce))
    }
}

struct Item3;

impl Item3 {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(A(0.0))
            .with(B(0.0))
            .with(C(0.0))
            .with(D(0.0))
    }
}

impl EntityMainComponent for Item3 {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(ab)).run(system!(cd)).run(system!(ce))
    }
}

struct Item4;

impl Item4 {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(A(0.0))
            .with(B(0.0))
            .with(C(0.0))
            .with(E(0.0))
    }
}

impl EntityMainComponent for Item4 {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(ab)).run(system!(cd)).run(system!(ce))
    }
}

fn ab(a: &mut A, b: &mut B) {
    std::mem::swap(&mut a.0, &mut b.0);
}

fn cd(c: &mut C, d: &mut D) {
    std::mem::swap(&mut c.0, &mut d.0);
}

fn ce(c: &mut C, e: &mut E) {
    std::mem::swap(&mut c.0, &mut e.0);
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
