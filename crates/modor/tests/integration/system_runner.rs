use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[action]
struct Action1;

#[action(Action1)]
struct Action2;

#[action(Action1, Action2)]
struct Action3;

struct Tester1 {
    run_system_ids: Arc<Mutex<Vec<u32>>>,
}

#[singleton]
impl Tester1 {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            run_system_ids: Arc::new(Mutex::new(vec![])),
        })
    }

    #[run_after(Action3)]
    fn run_after_action2(&self) {
        self.run_system_ids.lock().unwrap().push(1);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }

    #[run_as(Action3)]
    fn run_as_action3(&self) {
        self.run_system_ids.lock().unwrap().push(2);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }

    #[run_as(Action2)]
    fn run_as_action2(&self) {
        self.run_system_ids.lock().unwrap().push(3);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }

    #[run_as(Action1)]
    fn run_as_action1(&self) {
        self.run_system_ids.lock().unwrap().push(4);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }
}

struct Tester2 {
    run_system_ids: Arc<Mutex<Vec<u32>>>,
    first_system_run: AtomicBool,
}

#[singleton]
impl Tester2 {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            run_system_ids: Arc::new(Mutex::new(vec![])),
            first_system_run: AtomicBool::new(false),
        })
    }

    #[run_after_previous]
    fn run_after_previous_without_previous(&self) {
        self.first_system_run.store(true, Ordering::Release);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }

    #[run]
    fn run(&self) {
        self.run_system_ids.lock().unwrap().push(1);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }

    #[run_after_previous]
    fn run_after_previous(&self) {
        self.run_system_ids.lock().unwrap().push(2);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn run_tester1_and_tester2_sequentially() {
    let mut app: TestApp = App::new()
        .with_entity(Tester1::build())
        .with_entity(Tester2::build())
        .into();
    app.update();
    app.assert_singleton::<Tester1>()
        .has(|t: &Tester1| assert_eq!(*t.run_system_ids.lock().unwrap(), [4, 3, 2, 1]));
    app.assert_singleton::<Tester2>().has(|t: &Tester2| {
        assert_eq!(*t.run_system_ids.lock().unwrap(), [1, 2]);
        assert!(t.first_system_run.load(Ordering::Acquire));
    });
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_tester1_and_tester2_in_parallel() {
    modor_internal::retry!(10, {
        let mut app: TestApp = App::new()
            .with_thread_count(2)
            .with_entity(Tester1::build())
            .with_entity(Tester2::build())
            .into();
        let start = instant::Instant::now();
        app.update();
        app.assert_singleton::<Tester1>()
            .has(|t: &Tester1| assert_eq!(*t.run_system_ids.lock().unwrap(), [4, 3, 2, 1]));
        app.assert_singleton::<Tester2>().has(|t: &Tester2| {
            assert_eq!(*t.run_system_ids.lock().unwrap(), [1, 2]);
            assert!(t.first_system_run.load(Ordering::Acquire));
        });
        assert!(instant::Instant::now() - start < std::time::Duration::from_millis(250));
    });
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_tester1_in_parallel() {
    let mut app: TestApp = App::new()
        .with_thread_count(2)
        .with_entity(Tester1::build())
        .into();
    let start = instant::Instant::now();
    app.update();
    app.assert_singleton::<Tester1>()
        .has(|t: &Tester1| assert_eq!(*t.run_system_ids.lock().unwrap(), [4, 3, 2, 1]));
    assert!(instant::Instant::now() - start > std::time::Duration::from_millis(200));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_tester2_in_parallel() {
    let mut app: TestApp = App::new()
        .with_thread_count(2)
        .with_entity(Tester2::build())
        .into();
    let start = instant::Instant::now();
    app.update();
    app.assert_singleton::<Tester2>().has(|t: &Tester2| {
        assert_eq!(*t.run_system_ids.lock().unwrap(), [1, 2]);
        assert!(t.first_system_run.load(Ordering::Acquire));
    });
    assert!(instant::Instant::now() - start > std::time::Duration::from_millis(100));
}
