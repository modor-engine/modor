#[macro_use]
extern crate modor;

use modor::*;

fn main() {}

trait MyComponent {}

#[systems]
impl dyn MyComponent {}
//~^ error: only path types are supported (for example, `module::Type<GenericType>`)
