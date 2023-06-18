use modor::{App, Single, With};

#[derive(Component, Default)]
struct Tester {
    done_existing: bool,
    done_missing: bool,
}

#[systems]
impl Tester {
    #[run]
    fn run_existing(&mut self, number: Single<'_, Number>) {
        assert_eq!(number.0, 10);
        assert_eq!(number.entity().id(), 0);
        self.done_existing = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
    }

    #[run]
    fn run_other_existing(_: Option<Single<'_, Number>>) {
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
    }

    #[run]
    fn run_missing(&mut self, _: Single<'_, Other>) {
        self.done_missing = true;
    }
}

#[derive(SingletonComponent, NoSystem)]
struct Number(u32);

#[derive(SingletonComponent, NoSystem)]
struct Other(u32);

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_single() {
    App::new()
        .with_entity(Number(10))
        .with_entity(Tester::default())
        .with_entity(Tester::default())
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
fn run_systems_in_parallel() {
    modor_internal::retry!(60, {
        let start = instant::Instant::now();
        App::new()
            .with_thread_count(2)
            .with_entity(Number(10))
            .with_entity(Tester::default())
            .updated();
        assert!(start.elapsed() < std::time::Duration::from_millis(200));
    });
}
