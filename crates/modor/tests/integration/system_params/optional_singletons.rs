use modor::{App, Single, With};

#[derive(Component, Default)]
struct Tester {
    done_existing: bool,
    done_missing: bool,
}

#[systems]
impl Tester {
    #[run]
    fn run_existing(&mut self, number: Option<Single<'_, Number>>) {
        assert_eq!(number.map(|n| n.0), Some(10));
        self.done_existing = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
    }

    #[run]
    fn run_other_existing(_: Single<'_, Number>) {
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
    }

    #[run]
    fn run_missing(&mut self, other: Option<Single<'_, Other>>) {
        assert_eq!(other.map(|o| o.0), None);
        self.done_missing = true;
    }
}

#[derive(SingletonComponent, NoSystem)]
struct Number(u32);

#[derive(SingletonComponent, NoSystem)]
struct Other(u32);

#[modor_test]
fn use_single() {
    App::new()
        .with_entity(Number(10))
        .with_entity(Tester::default())
        .with_entity(Tester::default())
        .updated()
        .assert::<With<Tester>>(2, |e| {
            e.has(|t: &Tester| {
                assert!(t.done_existing);
                assert!(t.done_missing);
            })
        });
}

#[modor_test(disabled(wasm))]
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
