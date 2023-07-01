//! Tests performance of archetype iteration in systems with one system for all archetypes.
#![allow(unused_tuple_struct_fields)]

#[macro_use]
extern crate modor;

use criterion::{criterion_main, Criterion};
use modor::{App, BuiltEntity, EntityBuilder};

#[derive(Component)]
struct Data(f32);

#[systems]
impl Data {
    #[run]
    fn update(&mut self) {
        self.0 *= 2.0;
    }
}

macro_rules! create_entities {
    ($app:ident; $( $variants:ident ),*) => {
        $(
            #[derive(Component)]
            struct $variants(f32);

            #[systems]
            impl $variants {
                fn build() -> impl BuiltEntity {
                    EntityBuilder::new()
                        .component(Self(0.0))
                        .inherited(Data(1.0))
                }
            }

            for _ in 0..20 {
                $app = $app.with_entity($variants::build());
            }
        )*
    };
}

#[allow(clippy::items_after_statements, clippy::cognitive_complexity)]
fn run(c: &mut Criterion) {
    let mut app = App::new();
    create_entities!(app; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
    c.bench_function("one_system_fragmented_iteration", |b| {
        b.iter(|| app.update());
    });
}

mod group {
    criterion::criterion_group!(benches, super::run);
}
criterion_main!(group::benches);
