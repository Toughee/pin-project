#![feature(unsized_locals)]

use pin_project::pin_project;

#[pin_project(project_replace)] //~ ERROR E0277
struct Struct<T: ?Sized> {
    f: T,
}

#[pin_project(project_replace)] //~ ERROR E0277
struct TupleStruct<T: ?Sized>(T);

fn main() {}
