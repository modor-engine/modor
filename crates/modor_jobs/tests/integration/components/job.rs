use async_std::fs;
use modor::{App, Built, EntityBuilder, With};
use modor_jobs::{Job, JobPanickedError};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

struct FileSize {
    size: Result<Option<usize>, JobPanickedError>,
    should_poll: bool,
}

#[entity]
impl FileSize {
    fn build(path: impl Into<PathBuf>) -> impl Built<Self> {
        let path = path.into();
        EntityBuilder::new(Self {
            size: Ok(None),
            should_poll: true,
        })
        .with(Job::new(async { fs::read(path).await.unwrap().len() }))
    }

    #[run]
    fn poll(&mut self, job: &mut Job<usize>) {
        if !self.should_poll {
            return;
        }
        self.size = job.try_poll();
        self.should_poll = false;
    }
}

#[test]
fn run_successful_job() {
    App::new()
        .with_entity(FileSize::build(concat!("tests/assets/test.txt")))
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))))
        .with_update::<(), _>(|_: &mut FileSize| thread::sleep(Duration::from_millis(100)))
        .with_update::<(), _>(|s: &mut FileSize| s.should_poll = true)
        .updated()
        .assert::<With<FileSize>>(1, |e| {
            e.has(|s: &FileSize| assert_eq!(s.size, Ok(Some(12))))
        })
        .with_update::<(), _>(|s: &mut FileSize| s.should_poll = true)
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))));
}

#[test]
fn run_failing_job() {
    App::new()
        .with_entity(FileSize::build("not/existing/path"))
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))))
        .with_update::<(), _>(|_: &mut FileSize| thread::sleep(Duration::from_millis(100)))
        .with_update::<(), _>(|s: &mut FileSize| s.should_poll = true)
        .updated()
        .assert::<With<FileSize>>(1, |e| {
            e.has(|s: &FileSize| assert_eq!(s.size, Err(JobPanickedError)))
        })
        .with_update::<(), _>(|s: &mut FileSize| s.should_poll = true)
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))));
}
