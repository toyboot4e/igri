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

We want to disable developer UI on release build. Enable `dummy` feature flag to turn off `#[derive(Inspect)]` expansion.

> Be sure to out put other calls to `igri`, too!

