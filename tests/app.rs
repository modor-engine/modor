use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder, EntityMainComponent};

#[derive(PartialEq, Debug)]
struct Number(u32);

impl EntityMainComponent for Number {
    type Data = u32;

    fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
        builder.with_self(Self(data))
    }
}

#[derive(PartialEq, Debug)]
struct Text(String);

impl EntityMainComponent for Text {
    type Data = String;

    fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
        builder.with_self(Self(data))
    }
}

#[test]
fn configure_app() {
    let app = App::new()
        .with_thread_count(2)
        .with_entity::<Number>(10)
        .with_entity::<Text>("text".into());
    let mut app = TestApp::from(app);
    app.assert_entity(0)
        .has::<Number, _>(|c| assert_eq!(c, &Number(10)));
    app.assert_entity(1)
        .has::<Text, _>(|c| assert_eq!(c, &Text("text".into())));

    app.update();
    app.assert_entity(0)
        .has::<Number, _>(|c| assert_eq!(c, &Number(10)));
    app.assert_entity(1)
        .has::<Text, _>(|c| assert_eq!(c, &Text("text".into())));
}
