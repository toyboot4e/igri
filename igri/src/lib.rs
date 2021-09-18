/*!
ImGUI runtime inspector

```
use igri::Inspect;

#[derive(Inspect)]
pub struct MyCoolStruct<T> {
     xs: Vec<T>,
}
```


# `dummy` feature

We want to disable developer UI on release build. Enable `dummy` feature flag to turn off
`#[derive(Inspect)]` expansion.

> Be sure to opt out other calls to `igri`, too!

# Limitations

`Inspect` is a foreign trait from your code, which can only be implemented for types in your own
crate. So types in your upstream frameworks does not implement `Inspect`.

`igri` lets you tweak values via `imgui`, but it doesn't let you propagate the change (so for
example, your game view is not synced to changes caused via `igri`).

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
