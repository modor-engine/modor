use modor::{
    App, Changed, ChildBuilder, Entity, EntityBuilder, Filter, Or, Query, Single, SingleMut, With,
    World,
};
use std::panic::{RefUnwindSafe, UnwindSafe};

struct Singleton;

#[singleton]
impl Singleton {}

assert_impl_all!(App: Send, Unpin);
assert_impl_all!(Changed<u32>: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);
assert_impl_all!(ChildBuilder<'_>: Send, Unpin);
assert_impl_all!(Entity<'_>: Sync, Send, Unpin);
assert_impl_all!(EntityBuilder<Singleton, (), ()>: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);
assert_impl_all!(Filter<With<u32>>: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);
assert_impl_all!(Query<'_, ()>: Sync, Send, Unpin);
assert_impl_all!(Or<With<u32>>: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);
assert_impl_all!(Single<'_, Singleton>: Sync, Send, Unpin);
assert_impl_all!(SingleMut<'_, Singleton>: Sync, Send, Unpin);
assert_impl_all!(With<u32>: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);
assert_impl_all!(World<'_>: Sync, Send, Unpin);
