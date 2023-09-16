use modor::{systems, EntityMut, SingletonComponent};

#[derive(SingletonComponent)]
pub(crate) struct ResetEvent;

// TODO: create a derive macro for that
#[systems]
impl ResetEvent {
    #[run]
    fn remove(mut entity: EntityMut<'_>) {
        entity.delete();
    }
}
