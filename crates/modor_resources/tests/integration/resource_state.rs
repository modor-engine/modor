use modor_resources::{ResourceError, ResourceState};

#[modor::test]
fn retrieve_error_when_loading() {
    assert_eq!(ResourceState::Loading.error(), None);
}

#[modor::test]
fn retrieve_error_when_loaded() {
    assert_eq!(ResourceState::Loaded.error(), None);
}

#[modor::test]
fn retrieve_error_when_error() {
    let error = ResourceError::Other("error".into());
    assert_eq!(ResourceState::Error(error.clone()).error(), Some(&error));
}
