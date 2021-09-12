/*!
ImGUI runtime inspector

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
