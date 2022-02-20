use modor::testing::TestApp;
use modor::{system, Built, Entity, EntityBuilder, EntityMainComponent, Query, SystemRunner, With};

#[derive(Clone, Copy)]
struct AbsPosition(u16);

#[derive(Clone, Copy)]
struct RelPosition(u16);

struct Node;

impl EntityMainComponent for Node {
    type Data = u16;

    fn build(mut builder: EntityBuilder<'_, Self>, levels: Self::Data) -> Built {
        if levels > 0 {
            builder = builder.with_child::<Self>(levels - 1);
        }
        builder
            .with_option((levels % 2 == 1).then(|| RelPosition(levels)))
            .with_option((levels % 2 == 1).then(|| AbsPosition(levels)))
            .with_self(Self)
    }

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::update_absolute_positions))
    }
}

impl Node {
    fn update_absolute_positions(
        entities_with_pos: Query<'_, Entity<'_>, (With<AbsPosition>, With<RelPosition>)>,
        mut positions: Query<'_, (&mut AbsPosition, &RelPosition)>,
    ) {
        let mut entities: Vec<_> = entities_with_pos.iter().collect();
        entities.sort_unstable_by_key(|e| e.depth());
        for entity in entities {
            if let (Some((abs, rel)), Some((parent_abs, _))) =
                positions.get_with_first_parent_mut(entity.id())
            {
                abs.0 = parent_abs.0 + rel.0;
            }
        }
    }
}

#[test]
fn update_component_hierarchically() {
    let mut app = TestApp::new();
    let root_id = app.create_entity::<Node>(5);
    app.update();
    app.assert_entity(root_id)
        .has::<RelPosition, _>(|p| assert_eq!(p.0, 5))
        .has::<AbsPosition, _>(|p| assert_eq!(p.0, 5))
        .has_children(|c| {
            assert_eq!(c.len(), 1);
            app.assert_entity(c[0])
                .has_not::<RelPosition>()
                .has_not::<AbsPosition>()
                .has_children(|c| {
                    assert_eq!(c.len(), 1);
                    app.assert_entity(c[0])
                        .has::<RelPosition, _>(|p| assert_eq!(p.0, 3))
                        .has::<AbsPosition, _>(|p| assert_eq!(p.0, 8))
                        .has_children(|c| {
                            assert_eq!(c.len(), 1);
                            app.assert_entity(c[0])
                                .has_not::<RelPosition>()
                                .has_not::<AbsPosition>()
                                .has_children(|c| {
                                    assert_eq!(c.len(), 1);
                                    app.assert_entity(c[0])
                                        .has::<RelPosition, _>(|p| assert_eq!(p.0, 1))
                                        .has::<AbsPosition, _>(|p| assert_eq!(p.0, 9))
                                        .has_children(|c| {
                                            assert_eq!(c.len(), 1);
                                            app.assert_entity(c[0])
                                                .has_not::<RelPosition>()
                                                .has_not::<AbsPosition>()
                                                .has_children(|c| assert_eq!(c.len(), 0));
                                        });
                                });
                        });
                });
        });
}
