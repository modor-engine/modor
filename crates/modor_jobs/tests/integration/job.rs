use log::LevelFilter;
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
    fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            job: Job::new(async {
                async_std::task::sleep(Duration::from_millis(10)).await;
                async_std::fs::read(path).await.unwrap().len()
            }),
            size: Ok(None),
        }
    }

    #[run]
    fn poll(&mut self) {
        self.size = self.job.try_poll();
    }
}

#[test]
fn run_successful_job() {
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(FileLoader::new("tests/assets/test.txt"))
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

#[test]
fn run_failing_job() {
    App::new()
        .with_log_level(LevelFilter::Trace)
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
