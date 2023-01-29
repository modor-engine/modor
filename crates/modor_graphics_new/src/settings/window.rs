use modor::{Built, EntityBuilder};

#[derive(PartialEq, Eq, Clone)]
pub struct WindowTitle(pub(crate) String);

#[singleton]
impl WindowTitle {
    pub fn build(title: impl Into<String>) -> impl Built<Self> {
        EntityBuilder::new(Self(title.into()))
    }

    pub fn set(&mut self, title: impl Into<String>) {
        self.0 = title.into();
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct CursorVisibility {
    pub(crate) is_visible: bool,
}

#[singleton]
impl CursorVisibility {
    pub fn build(is_visible: bool) -> impl Built<Self> {
        EntityBuilder::new(Self { is_visible })
    }

    pub fn set(&mut self, is_visible: bool) {
        self.is_visible = is_visible;
    }
}
