use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder, EntityMainComponent, Global};

#[derive(PartialEq, Debug)]
struct Integer(u32);

impl EntityMainComponent for Integer {
    type Type = ();
    type Data = u32;

    fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built<'_> {
        builder.with_self(Self(data))
    }
}

#[derive(PartialEq, Debug)]
struct Text(String);

impl EntityMainComponent for Text {
    type Type = ();
    type Data = String;

    fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built<'_> {
        builder.with_self(Self(data))
    }
}

struct Char(char);

impl Global for Char {}

#[test]
fn configure_app() {
    let app = App::new()
        .with_thread_count(2)
        .with_entity::<Integer>(10)
        .with_entity::<Text>("text".into())
        .with_global(Char('c'));
    let mut app = TestApp::from(app);
    app.assert_entity(0)
        .has::<Integer, _>(|i| assert_eq!(i, &Integer(10)));
    app.assert_entity(1)
        .has::<Text, _>(|t| assert_eq!(t, &Text("text".into())));
    app.assert_global_exists::<Char, _>(|f| assert_eq!(f.0, 'c'));

    app.update();
    app.assert_entity(0)
        .has::<Integer, _>(|i| assert_eq!(i, &Integer(10)));
    app.assert_entity(1)
        .has::<Text, _>(|t| assert_eq!(t, &Text("text".into())));
    app.assert_global_exists::<Char, _>(|f| assert_eq!(f.0, 'c'));
}
