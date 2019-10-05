// compile-fail

#![feature(optin_builtin_traits)]
#![feature(trivial_bounds)]

#[macro_use]
extern crate auxiliary_macros;

use pin_project::pin_project;

#[pin_project] //~ ERROR pattern does not mention field `__field`
#[add_pinned_field]
struct Foo {
    #[pin]
    field: u32,
}

fn main() {}
