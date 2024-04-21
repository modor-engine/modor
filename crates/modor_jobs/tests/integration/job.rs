use modor_jobs::{Job, JobPanickedError};
use std::thread;
use std::time::Duration;

#[modor::test(disabled(wasm))]
fn run_successful_job() {
    let mut job = Job::new(file_size("assets/test.txt"));
    let result = retrieve_result(&mut job);
    assert_eq!(result, Ok(Some(12)));
    assert_eq!(job.try_poll(), Ok(None));
}

#[modor::test(disabled(wasm))]
fn run_failing_job() {
    let mut job = Job::new(file_size("not/existing/path"));
    let result = retrieve_result(&mut job);
    assert_eq!(result, Err(JobPanickedError));
    assert_eq!(job.try_poll(), Ok(None));
}

#[allow(unused_variables, clippy::unused_async)]
async fn file_size(path: &str) -> usize {
    #[cfg(not(target_arch = "wasm32"))]
    {
        async_std::task::sleep(Duration::from_millis(10)).await;
        async_std::fs::read(path).await.unwrap().len()
    }
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
}

fn retrieve_result(job: &mut Job<usize>) -> Result<Option<usize>, JobPanickedError> {
    const MAX_RETRIES: u32 = 100;
    for _ in 0..MAX_RETRIES {
        thread::sleep(Duration::from_millis(10));
        let result = job.try_poll();
        if result != Ok(None) {
            return result;
        }
    }
    panic!("max retries reached");
}
