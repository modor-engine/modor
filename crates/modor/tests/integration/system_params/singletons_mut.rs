use modor::{App, Built, EntityBuilder, SingleMut, With};

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
    fn run_existing(&mut self, number: SingleMut<'_, Number>) {
        assert_eq!(number.0, 10);
        assert_eq!(number.entity().id(), 0);
        self.done_existing = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
    }

    #[run]
    fn run_other_existing(_: SingleMut<'_, Number>) {
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
    }

    #[run]
    fn run_missing(&mut self, _: SingleMut<'_, Other>) {
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
fn use_single_mut() {
    App::new()
        .with_entity(Number::build(10))
        .with_entity(Tester::build())
        .with_entity(Tester::build())
        .updated()
        .assert::<With<Tester>>(2, |e| {
            e.has(|t: &Tester| {
                assert!(t.done_existing);
                assert!(!t.done_missing);
            })
        });
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_must_use)]
fn run_systems_in_parallel() {
    let start = instant::Instant::now();
    App::new()
        .with_thread_count(2)
        .with_entity(Number::build(10))
        .with_entity(Tester::build())
        .updated();
    assert!(instant::Instant::now() - start > std::time::Duration::from_millis(200));
}
