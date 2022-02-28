//! Tests performance of archetype iteration in systems with one system per archetype.

use criterion::{criterion_main, Criterion};
use modor::{system, App, Built, EntityBuilder, EntityMainComponent, SystemRunner};

struct Data(f32);

macro_rules! create_entities {
    ($app:ident; $( $variants:ident ),*) => {
        $(
            struct $variants(f32);

            impl EntityMainComponent for $variants {
                type Type = ();
                type Data = ();

                fn build(builder: EntityBuilder<'_, Self>, _: Self::Data) -> Built<'_> {
                    builder.with(Data(1.0)).with_self(Self(0.0))
                }

                fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
                    runner.run(system!(Self::update))
                }
            }

            impl $variants {
                fn update(data: &mut Data) {
                    data.0 *= 2.0;
                }
            }

            for _ in 0..20 {
                $app = $app.with_entity::<$variants>(());
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
