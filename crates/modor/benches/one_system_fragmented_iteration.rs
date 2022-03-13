//! Tests performance of archetype iteration in systems with one system for all archetypes.

#[macro_use]
extern crate modor;

use criterion::{criterion_main, Criterion};
use modor::{App, Built, EntityBuilder};

struct Data(f32);

#[entity]
impl Data {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(1.0))
    }

    #[run]
    fn update(&mut self) {
        self.0 *= 2.0;
    }
}

macro_rules! create_entities {
    ($app:ident; $( $variants:ident ),*) => {
        $(
            struct $variants(f32);

            #[entity]
            impl $variants {
                fn build() -> impl Built<Self> {
                    EntityBuilder::new(Self(0.0))
                        .inherit_from(Data::build())
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
