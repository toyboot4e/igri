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

