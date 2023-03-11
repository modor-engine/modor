use modor::{App, With};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Action)]
struct Action1;

#[derive(Action)]
struct Action2(Action1);

#[derive(Action)]
struct Action3(Action1, Action2);

#[derive(SingletonComponent, Default)]
struct Tester1 {
    run_system_ids: Arc<Mutex<Vec<u32>>>,
}

#[systems]
impl Tester1 {
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

#[derive(SingletonComponent, Default)]
struct Tester2 {
    run_system_ids: Arc<Mutex<Vec<u32>>>,
    first_system_run: AtomicBool,
}

#[systems]
impl Tester2 {
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

#[derive(SingletonComponent, Default)]
struct Tester3 {
    run_system_ids: Arc<Mutex<Vec<u32>>>,
}

#[systems]
impl Tester3 {
    #[run_as(Action1)]
    fn run(&self) {
        self.run_system_ids.lock().unwrap().push(1);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }

    #[run_after_previous_and(Action2)]
    fn run_after_previous_and(&self) {
        self.run_system_ids.lock().unwrap().push(2);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }

    #[run_as(Action2)]
    fn run_as(&self) {
        self.run_system_ids.lock().unwrap().push(3);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn run_tester1_and_tester2_and_tester3_sequentially() {
    App::new()
        .with_entity(Tester1::default())
        .with_entity(Tester2::default())
        .with_entity(Tester3::default())
        .updated()
        .assert::<With<Tester1>>(1, |e| {
            e.has(|t: &Tester1| assert_eq!(*t.run_system_ids.lock().unwrap(), [4, 3, 2, 1]))
        })
        .assert::<With<Tester2>>(1, |e| {
            e.has(|t: &Tester2| {
                assert_eq!(*t.run_system_ids.lock().unwrap(), [1, 2]);
                assert!(t.first_system_run.load(Ordering::Acquire));
            })
        })
        .assert::<With<Tester3>>(1, |e| {
            e.has(|t: &Tester3| assert_eq!(*t.run_system_ids.lock().unwrap(), [1, 3, 2]))
        });
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_tester1_and_tester2_in_parallel() {
    modor_internal::retry!(10, {
        let start = instant::Instant::now();
        App::new()
            .with_thread_count(2)
            .with_entity(Tester1::default())
            .with_entity(Tester2::default())
            .updated()
            .assert::<With<Tester1>>(1, |e| {
                e.has(|t: &Tester1| assert_eq!(*t.run_system_ids.lock().unwrap(), [4, 3, 2, 1]))
            })
            .assert::<With<Tester2>>(1, |e| {
                e.has(|t: &Tester2| {
                    assert_eq!(*t.run_system_ids.lock().unwrap(), [1, 2]);
                    assert!(t.first_system_run.load(Ordering::Acquire));
                })
            });
        assert!(start.elapsed() < std::time::Duration::from_millis(250));
    });
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_tester1_in_parallel() {
    let start = instant::Instant::now();
    App::new()
        .with_thread_count(2)
        .with_entity(Tester1::default())
        .updated()
        .assert::<With<Tester1>>(1, |e| {
            e.has(|t: &Tester1| assert_eq!(*t.run_system_ids.lock().unwrap(), [4, 3, 2, 1]))
        });
    assert!(start.elapsed() > std::time::Duration::from_millis(200));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_tester2_in_parallel() {
    let start = instant::Instant::now();
    App::new()
        .with_thread_count(2)
        .with_entity(Tester2::default())
        .updated()
        .assert::<With<Tester2>>(1, |e| {
            e.has(|t: &Tester2| {
                assert_eq!(*t.run_system_ids.lock().unwrap(), [1, 2]);
                assert!(t.first_system_run.load(Ordering::Acquire));
            })
        });
    assert!(start.elapsed() > std::time::Duration::from_millis(100));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_tester3_in_parallel() {
    let start = instant::Instant::now();
    App::new()
        .with_thread_count(2)
        .with_entity(Tester3::default())
        .updated()
        .assert::<With<Tester3>>(1, |e| {
            e.has(|t: &Tester3| assert_eq!(*t.run_system_ids.lock().unwrap(), [1, 3, 2]))
        });
    assert!(start.elapsed() > std::time::Duration::from_millis(150));
}
