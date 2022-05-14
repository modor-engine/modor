use instant::Instant;
use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder, Single};
use std::time::Duration;

struct Tester {
    done_existing: bool,
    done_missing: bool,
}

#[entity]
impl Tester {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            done_existing: false,
            done_missing: false,
        })
    }

    #[run]
    fn run_existing(&mut self, number: Single<'_, Number>) {
        assert_eq!(number.0, 10);
        self.done_existing = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(Duration::from_millis(100));
    }

    #[run]
    fn run_other_existing(_: Option<Single<'_, Number>>) {
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(Duration::from_millis(100));
    }

    #[run]
    fn run_missing(&mut self, _: Single<'_, Other>) {
        self.done_missing = true;
    }
}

struct Number(u32);

#[singleton]
impl Number {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(value))
    }
}

struct Other(u32);

#[singleton]
impl Other {}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_single() {
    let mut app: TestApp = App::new().with_entity(Number::build(10)).into();
    let tester1_id = app.create_entity(Tester::build());
    let tester2_id = app.create_entity(Tester::build());
    app.update();
    app.assert_entity(tester1_id).has(|t: &Tester| {
        assert!(t.done_existing);
        assert!(!t.done_missing);
    });
    app.assert_entity(tester2_id).has(|t: &Tester| {
        assert!(t.done_existing);
        assert!(!t.done_missing);
    });
}

#[test]
fn run_systems_in_parallel() {
    modor_internal::retry!(10, {
        let mut app: TestApp = App::new()
            .with_thread_count(2)
            .with_entity(Number::build(10))
            .with_entity(Tester::build())
            .into();
        let start = Instant::now();
        app.update();
        assert!(Instant::now() - start < Duration::from_millis(150))
    });
}
