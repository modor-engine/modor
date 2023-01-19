use modor::{App, Built, EntityBuilder, With};

#[derive(Action)]
struct Action1;

#[derive(Action)]
struct Action2(Action1);

#[derive(Action)]
struct Action3(Action2);

struct Runner {
    run_system_ids: Vec<u32>,
}

#[entity]
impl Runner {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            run_system_ids: vec![],
        })
    }

    #[run_as(Action3)]
    fn run_action_3(&mut self) {
        self.run_system_ids.push(3);
    }

    #[run_as(Action1)]
    fn run_action_1(&mut self) {
        self.run_system_ids.push(1);
    }
}

#[test]
fn run_systems_in_order_with_never_used_action_dependency_between_two_systems() {
    App::new()
        .with_entity(Runner::build())
        .updated()
        .assert::<With<Runner>>(1, |e| {
            e.has(|r: &Runner| assert_eq!(r.run_system_ids, [1, 3]))
        });
}