use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder, EntityMainComponent};

#[derive(PartialEq, Debug)]
struct Integer(u32);

impl Integer {
    fn build(integer: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(integer))
    }
}

impl EntityMainComponent for Integer {
    type Type = ();
}

#[derive(PartialEq, Debug)]
struct Text(String);

impl Text {
    fn build(text: String) -> impl Built<Self> {
        EntityBuilder::new(Self(text))
    }
}

impl EntityMainComponent for Text {
    type Type = ();
}

#[test]
fn configure_app() {
    let app = App::new()
        .with_thread_count(2)
        .with_entity(Integer::build(10))
        .with_entity(Text::build("text".into()));
    let mut app = TestApp::from(app);
    app.assert_entity(0)
        .has::<Integer, _>(|i| assert_eq!(i, &Integer(10)));
    app.assert_entity(1)
        .has::<Text, _>(|t| assert_eq!(t, &Text("text".into())));

    app.update();
    app.assert_entity(0)
        .has::<Integer, _>(|i| assert_eq!(i, &Integer(10)));
    app.assert_entity(1)
        .has::<Text, _>(|t| assert_eq!(t, &Text("text".into())));
}
