#[macro_use]
extern crate modor;

use modor::*;

fn main() {}

trait MyEntity {}

#[entity]
impl dyn MyEntity {}
//~^ error: only path types are supported (for example, `module::Type<GenericType>`)
