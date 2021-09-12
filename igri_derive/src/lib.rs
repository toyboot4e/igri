#[cfg(not(feature = "dummy"))]
mod inspect;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/**
Derive `Inspect` trait. Requires `igri` to be in scope

# Attributes

TODO
*/
#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn inspect(input: TokenStream) -> TokenStream {
    // create empty implementation on dummy feature
    #[cfg(feature = "dummy")]
    {
        TokenStream::new()
    }

    // create implementation if it's not dummmy
    #[cfg(not(feature = "dummy"))]
    {
        let ast = parse_macro_input!(input as DeriveInput);
        TokenStream::from(inspect::impl_inspect(ast))
    }
}
