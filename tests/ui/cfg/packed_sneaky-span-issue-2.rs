use auxiliary_macros::hidden_repr;
use pin_project::pin_project;

//~ ERROR may not be used on #[repr(packed)] types
// span is lost.
// Refs: https://github.com/rust-lang/rust/issues/43081
#[pin_project]
#[hidden_repr(packed)]
struct S {
    #[cfg(any())]
    #[pin]
    f: u32,
    #[cfg(not(any()))]
    #[pin]
    f: u8,
}

fn main() {}
