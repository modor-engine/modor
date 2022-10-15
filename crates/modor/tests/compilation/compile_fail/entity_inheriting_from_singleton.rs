#[macro_use]
extern crate modor;

use modor::*;

fn main() {}

struct MySingleton;

#[singleton]
impl MySingleton {
    #[allow(unused)]
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }
}

struct MyEntity;

#[entity]
impl MyEntity {
    #[allow(unused)]
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(MySingleton::build())
        //~^ error: type mismatch resolving `<MyEntity as EntityMainComponent>::Type == Singleton`
    }
}
