use modor::{SingletonComponent, TemporaryComponent};

#[derive(SingletonComponent, TemporaryComponent)]
pub(crate) struct ResetEvent;
