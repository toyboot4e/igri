//! `igri` tests
//!
//! See `igri-demo` for GUI test.
//!
//! Run `cargo expand --test derive` to see macro-expaned code (that's why this is an integrated
//! test).

use igri::Inspect;

#[test]
fn derive() {
    #[derive(Inspect)]
    pub struct Struct {
        x: f32,
        y: u32,
    }

    #[derive(Inspect)]
    pub struct Tuple1(f32);

    #[derive(Inspect)]
    pub struct Tuple2(f32, [u32; 2]);

    #[derive(Inspect)]
    pub struct Unit;

    #[derive(Inspect)]
    enum Plain {
        A,
        B,
        C,
    }

    #[derive(Inspect)]
    enum Wrapper {
        A(usize),
        B(f32),
        C(String),
    }

    #[derive(Inspect)]
    enum Complex {
        Struct { x: f32, u: u32 },
        Tuple(f32, u32),
        Unit,
    }
}
