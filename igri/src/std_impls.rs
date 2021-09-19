/*!
`paste::paste!` concats identifiers in declarative macro with `[< .. >]` syntax
*/

// TODO: support more types

use std::{
    borrow::Cow,
    cell::Cell,
    collections::{LinkedList, VecDeque},
    marker::PhantomData,
    num::*,
    ops::DerefMut,
    path::PathBuf,
    time::{Duration, Instant},
};

use imgui::Ui;

use crate::Inspect;

// primitives

macro_rules! im_ui_method {
    ($ty:ident, $method:ident) => {
        impl Inspect for $ty {
            fn inspect(&mut self, ui: &$crate::imgui::Ui, label: &str) {
                let _changed = ui.$method(label, self);
            }
        }
    };
}

im_ui_method!(bool, checkbox);

impl Inspect for String {
    fn inspect(&mut self, ui: &imgui::Ui, label: &str) {
        // FIXME: Consider supporting multiline text
        // https://docs.rs/imgui/latest/imgui/struct.InputTextMultiline.html
        let _changed = ui.input_text(label, self).build();
    }
}

// TODO: char?

macro_rules! im_input {
    ($ty:ident, $as:ty, $method:ident) => {
        impl Inspect for $ty {
            fn inspect(&mut self, ui: &$crate::imgui::Ui, label: &str) {
                let mut x = *self as $as;
                if ui.$method(format!("{}", label), &mut x).build() {
                    *self = x as $ty;
                }
            }
        }
    };
}

macro_rules! impl_array {
    ($ty:ty, $N:expr, $as:ty, $method:ident) => {
        impl Inspect for [$ty; $N] {
            #[allow(warnings)]
            fn inspect(&mut self, ui: &$crate::imgui::Ui, label: &str) {
                // FIXME: stable rust support
                // use arraytools::ArrayTools;
                let mut xs = self.map(|x| x as $as);
                let label = format!("{}", label);
                if ui.$method(label, &mut xs).build() {
                    *self = xs.map(|x| x as $ty);
                }
            }
        }
    };
    ($ty:ty, $as:ty, $method:ident) => {
        paste::paste! {
            im_input!($ty, $as, $method);
            impl_array!($ty, 2, $as, [<$method 2>]);
            impl_array!($ty, 3, $as, [<$method 3>]);
            impl_array!($ty, 4, $as, [<$method 4>]);
        }
    };
}

impl_array!(f32, f32, input_float);
impl_array!(f64, f32, input_float);

impl_array!(i8, i32, input_int);
impl_array!(i16, i32, input_int);
impl_array!(i32, i32, input_int);
impl_array!(i64, i32, input_int);

impl_array!(u8, i32, input_int);
impl_array!(u16, i32, input_int);
impl_array!(u32, i32, input_int);
impl_array!(u64, i32, input_int);

impl_array!(isize, i32, input_int);
impl_array!(usize, i32, input_int);

/// impl Inspect for `(T0, T1, ..)`
macro_rules! impl_tuple {
    ($($i:expr),*) => {
        paste::paste! {
            impl<$([<T $i>]),*> Inspect for ($([<T $i>]),*)
            where
                $([<T $i>]: Inspect,)*
            {
                fn inspect(&mut self, ui: &Ui, label: &str) {
                    $crate::nest(ui, label, || {
                        $(
                            self.$i.inspect(ui, stringify!($i));
                        )*
                    });
                }
            }
        }
    };
}

impl_tuple!(0, 1);
impl_tuple!(0, 1, 2);
impl_tuple!(0, 1, 2, 3);

// non-zero types

macro_rules! impl_non_zero {
    ($ty:ident) => {
        impl Inspect for $ty {
            fn inspect(&mut self, ui: &Ui, label: &str) {
                let mut x = self.clone().get();
                x.inspect(ui, label);
                if let Some(new_value) = $ty::new(x) {
                    *self = new_value;
                }
            }
        }
    };
    ($($ty:ident),* $(,)?) => {
        $(
            impl_non_zero!($ty);
        )*
    };
}

impl_non_zero!(
    NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64,
);

// None

impl<T> Inspect for [T; 0] {
    fn inspect(&mut self, _ui: &Ui, _label: &str) {}
}

impl<T: Inspect> Inspect for Option<T> {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        match self {
            Some(x) => x.inspect(ui, label),
            // FIXME: selectable enum
            None => ui.label_text(format!("{}", label), "None"),
        }
    }
}

impl<T> Inspect for PhantomData<T> {
    fn inspect(&mut self, _ui: &Ui, _label: &str) {}
}

// Wrappers

impl<'a, T: std::borrow::ToOwned + ?Sized> Inspect for Cow<'a, T>
where
    T::Owned: Inspect,
{
    fn inspect(&mut self, ui: &Ui, label: &str) {
        self.to_mut().inspect(ui, label);
    }
}

impl<T: Inspect + ?Sized> Inspect for Box<T> {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        self.deref_mut().inspect(ui, label);
    }
}

impl<T: Inspect + Copy> Inspect for Cell<T> {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let mut x = self.get();
        x.inspect(ui, label);
        self.set(x);
    }
}

// collections

macro_rules! impl_seq {
    ($ty:ident) => {
        impl<T: Inspect> Inspect for $ty<T> {
            fn inspect(&mut self, ui: &Ui, label: &str) {
                crate::seq(self.iter_mut(), ui, label);
            }
        }
    };
    ($($ty:ident),* $(,)?) => {
        $(
            impl_seq!($ty);
        )*
    };
}

impl_seq!(Vec, VecDeque, LinkedList);

// more std types

impl Inspect for Duration {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let time = self.as_secs_f32();
        ui.label_text(label, format!("{}", time));
    }
}

impl Inspect for Instant {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        ui.label_text(label, format!("{:?}", self));
    }
}

impl Inspect for PathBuf {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let mut s = format!("{:?}", self);
        if ui.input_text(label, &mut s).build() {
            *self = PathBuf::from(s);
        }
    }
}
