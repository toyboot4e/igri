# ImGUI runtime inspector

`igri` is a runtime inspector powered by [imgui-rs]

[imgui-rs]: https://github.com/imgui-rs/imgui-rs

```rust
use igri::Inspect;

#[derive(Inspect)]
pub struct MyCoolStruct<T> {
     xs: Vec<T>,
}
```

## `dummy` feature

We don't need developer UI on release. `#[derive(Inspect)]` expansion can optionally be turned off if `dumyy` feature flag is specified.

> Be sure to out put other calls to `igri`, too!

