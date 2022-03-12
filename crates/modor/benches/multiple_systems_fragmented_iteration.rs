//! Tests performance of archetype iteration in systems with one system per archetype.

use criterion::{criterion_main, Criterion};
use modor::{system, App, Built, EntityBuilder, EntityMainComponent, SystemRunner};

struct Data(f32);

macro_rules! create_entities {
    ($app:ident; $( $variants:ident ),*) => {
        $(
            struct $variants(f32);

            impl $variants {
                fn build() -> impl Built<Self> {
                    EntityBuilder::new(Self(0.0)).with(Data(1.0))
                }

                fn update(data: &mut Data) {
                    data.0 *= 2.0;
                }
            }

            impl EntityMainComponent for $variants {
                type Type = ();

                fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
                    runner.run(system!(Self::update))
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
    c.bench_function("multiple_systems_fragmented_iteration", |b| {
        b.iter(|| app.update());
    });
}

mod group {
    criterion::criterion_group!(benches, super::run);
}
criterion_main!(group::benches);
