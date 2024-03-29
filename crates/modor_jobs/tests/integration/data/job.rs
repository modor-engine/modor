use modor::{App, With};
use modor_jobs::{Job, JobPanickedError};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Component)]
struct FileLoader {
    job: Job<usize>,
    size: Result<Option<usize>, JobPanickedError>,
}

#[systems]
impl FileLoader {
    #[allow(unused_variables)]
    fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            job: Job::new(async {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    async_std::task::sleep(Duration::from_millis(10)).await;
                    async_std::fs::read(path).await.unwrap().len()
                }
                #[cfg(target_arch = "wasm32")]
                {
                    0
                }
            }),
            size: Ok(None),
        }
    }

    #[run]
    fn poll(&mut self) {
        self.size = self.job.try_poll();
    }
}

#[modor_test(disabled(wasm))]
fn run_successful_job() {
    App::new()
        .with_entity(FileLoader::new("assets/test.txt"))
        .updated_until_all::<(), _>(Some(100), |l: &FileLoader| {
            thread::sleep(Duration::from_millis(10));
            l.size != Ok(None)
        })
        .assert::<With<FileLoader>>(1, |e| {
            e.has(|l: &FileLoader| assert_eq!(l.size, Ok(Some(12))))
        })
        .updated()
        .assert::<With<FileLoader>>(1, |e| e.has(|l: &FileLoader| assert_eq!(l.size, Ok(None))));
}

#[modor_test(disabled(wasm))]
fn run_failing_job() {
    App::new()
        .with_entity(FileLoader::new("not/existing/path"))
        .updated_until_all::<(), _>(Some(100), |l: &FileLoader| {
            thread::sleep(Duration::from_millis(10));
            l.size != Ok(None)
        })
        .assert::<With<FileLoader>>(1, |e| {
            e.has(|l: &FileLoader| assert_eq!(l.size, Err(JobPanickedError)))
        })
        .updated()
        .assert::<With<FileLoader>>(1, |e| e.has(|l: &FileLoader| assert_eq!(l.size, Ok(None))));
}

#[modor_test(disabled(wasm))]
fn drop_not_finished_job() {
    App::new()
        .with_entity(FileLoader::new("assets/test.txt"))
        .with_deleted_entities::<With<FileLoader>>()
        .updated();
    thread::sleep(Duration::from_millis(100));
}
