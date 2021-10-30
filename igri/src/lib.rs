/*!
ImGUI runtime inspector

```
use igri::Inspect;

#[derive(Inspect)]
pub struct MyCoolStruct<T> {
     xs: Vec<T>,
}
```

# `enum` support

`#[derive(Inspect)]` for `enum` by default is implemented as a tag selector + variant field
inspectors. On the tag switch, the inspected value is replaced with the target variant with default
values. **It requires every field to implement `Default`**.

If you specify `#[inspect(no_tag)]` attribute, the tag switcher is disabled and the `Default`
implementation is not needed.

# `Inspect` impls for `std` types

`Option<T>::inspect` comes with the tag selector, which requires `T: Default` boundary. If it's too
restrictive, you can override using `#[inspect(with = "<function>")]` atttribute.

# Supported attributes (`#[inspect(attribute = value)]`)

| attribute             | over          | description                                                                           |
|---                    |---            |---                                                                                    |
| `skip`                | type or field | Skip inspection                                                                       |
| `with = "<function>"` | type or field | Override the inspection with the function                                             |
| `as  = "<Type>"`      | type or field | Inspect as the type, converting with `Into`                                           |
| `open = <bool>`       | type          | If the node is open by default                                                        |
| `bounds = "<bounds>"` | type          | Override `where` boundary for the `Inspect` impl (default: each `FieldType: Inspect`) |
| `no_tag`              | type (`enum`) | Disable tag selector (see `enum` support for more info)                               |

# `dummy` feature

We want to disable developer UI on release build. Enable `dummy` feature flag to turn off
`#[derive(Inspect)]` expansion.

> Be sure to opt out other calls to `igri`, too!

# Limitations

`Inspect` is a foreign trait from your code, and it can only be implemented for types in your own
crate. So types in your upstream framework might not implement `Inspect`.

`igri` lets you tweak values via `imgui`, but it doesn't let you propagate the change. For example,
your game view would not be synced to changes made with `igri`.
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

// Derive macro can have same name as trait
pub use igri_derive::Inspect;
