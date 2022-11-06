use log::LevelFilter;
use modor::{App, Built, EntityBuilder, With};
use modor_jobs::{Job, JobPanickedError};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

struct FileSize {
    size: Result<Option<usize>, JobPanickedError>,
}

#[entity]
impl FileSize {
    fn build(path: impl Into<PathBuf>) -> impl Built<Self> {
        let path = path.into();
        EntityBuilder::new(Self { size: Ok(None) }).with(Job::new(async {
            async_std::fs::read(path).await.unwrap().len()
        }))
    }

    #[run]
    fn poll(&mut self, job: &mut Job<usize>) {
        self.size = job.try_poll();
    }
}

#[test]
fn run_successful_job() {
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(FileSize::build(concat!("tests/assets/test.txt")))
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
        .with_entity(FileSize::build("not/existing/path"))
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
