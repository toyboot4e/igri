//! `igri` tests
//!
//! See `igri-demo` for GUI test.
//!
//! Run `cargo expand --test derive` to see macro-expaned code (that's why this is an integrated
//! test).

use igri::Inspect;

fn f(x: &mut f32, ui: &imgui::Ui, label: &str) {
    ui.label_text(label, format!("{}", x));
}

#[test]
fn derive() {
    #[derive(Inspect)]
    pub struct Struct {
        #[inspect(with = "f")]
        x: f32,
        y: u32,
    }

    #[derive(Inspect)]
    pub struct Tuple1(#[inspect(with = "f")] f32);

    #[derive(Inspect)]
    pub struct Tuple2(f32, [u32; 2]);

    #[derive(Inspect)]
    pub struct Unit;

    #[derive(Debug, Clone, PartialEq, Inspect)]
    pub struct VirtualUnitA {
        #[inspect(skip)]
        hidden: f32,
    }

    #[derive(Debug, Clone, PartialEq, Inspect)]
    pub struct VirtualUnitB(#[inspect(skip)] f32);

    #[derive(Inspect)]
    enum Plain {
        A,
        B,
        C,
    }

    #[derive(Inspect)]
    enum Wrapper {
        A(usize),
        B(#[inspect(with = "f")] f32),
        C(String),
    }

    #[derive(Inspect)]
    enum Complex {
        Struct { x: f32, u: u32 },
        Tuple(f32, u32),
        Unit,
    }
}
