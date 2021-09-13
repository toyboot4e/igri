use crate as igri;
use igri_derive::Inspect;

#[test]
fn derive() {
    #[derive(Inspect)]
    struct X {
        x: u32,
        y: String,
        #[inspect(skip)]
        z: Vec<()>,
    }

    #[derive(Clone, PartialEq, Inspect)]
    enum Unit {
        A,
        B,
        C,
        D,
        E,
        F,
        G,
    }

    #[derive(Clone, PartialEq, Inspect)]
    enum Complex {
        Unit,
        Tuple(u32, u32),
        // Fields { x: u32, y: u32 },
    }
}
