/*!
ImGUI runtime inspector

# Limitations

`Inspect` is a foreign trait from your code. Foreign traits can only be implemented for types in
your crate.

# `dummy` feature

We don't need developer UI on release. `#[derive(Inspect)]` expansion can optionally be turned off if `dumyy` feature flag is specified.

> Be sure to put other calls to `igri` in `#[cfg(debug_assertions)]` branch, too!

* TODO: enum tag + field settings
* TODO: delegate
*/

#[cfg(not(feature = "dummy"))]
pub extern crate imgui;

#[cfg(not(feature = "dummy"))]
mod inspect;

#[cfg(not(feature = "dummy"))]
#[cfg(debug_assertions)]
pub use inspect::*;

#[cfg(not(feature = "dummy"))]
mod std_impls;

#[cfg(test)]
mod test;

// Derive macro can have same name as trait
pub use igri_derive::Inspect;
