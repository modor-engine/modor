use modor::{App, With};

#[derive(SingletonComponent, TemporaryComponent)]
struct MyComponent;

#[modor_test]
fn create_temporary_component() {
    App::new()
        .with_entity(MyComponent)
        .assert::<With<MyComponent>>(1, |e| e)
        .updated()
        .assert::<With<MyComponent>>(0, |e| e);
}
