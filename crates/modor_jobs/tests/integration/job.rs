use log::LevelFilter;
use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_jobs::{Job, JobPanickedError};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Component)]
struct FileLoader {
    job: Job<usize>,
}

#[systems]
impl FileLoader {
    fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            job: Job::new(async {
                async_std::task::sleep(Duration::from_millis(10)).await;
                async_std::fs::read(path).await.unwrap().len()
            }),
        }
    }

    #[run]
    fn poll(&mut self, size: &mut FileSize) {
        size.size = self.job.try_poll();
    }
}

#[derive(Component)]
struct FileSize {
    size: Result<Option<usize>, JobPanickedError>,
}

#[systems]
impl FileSize {
    fn new() -> Self {
        Self { size: Ok(None) }
    }
}

fn file(path: &str) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(FileLoader::new(path))
        .with(FileSize::new())
}

#[test]
fn run_successful_job() {
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(file("tests/assets/test.txt"))
        .updated_until_all::<(), _>(Some(100), |s: &FileSize| {
            thread::sleep(Duration::from_millis(10));
            s.size != Ok(None)
        })
        .assert::<With<FileSize>>(1, |e| {
            e.has(|s: &FileSize| assert_eq!(s.size, Ok(Some(12))))
        })
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))));
}

#[test]
fn run_failing_job() {
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(file("not/existing/path"))
        .updated_until_all::<(), _>(Some(100), |s: &FileSize| {
            thread::sleep(Duration::from_millis(10));
            s.size != Ok(None)
        })
        .assert::<With<FileSize>>(1, |e| {
            e.has(|s: &FileSize| assert_eq!(s.size, Err(JobPanickedError)))
        })
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))));
}
