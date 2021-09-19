use imgui::Ui;

/// ImGUI runtime inspection
pub trait Inspect {
    // mutable reference only
    fn inspect(&mut self, ui: &Ui, label: &str);
}

/// Inspects a sequence of inspectable items
pub fn seq<'a, T: Inspect + 'a>(xs: impl Iterator<Item = &'a mut T>, ui: &Ui, label: &str) {
    self::nest(ui, label, || {
        use std::fmt::Write;
        let mut buf = String::with_capacity(2);

        for (i, x) in xs.enumerate() {
            buf.clear();
            write!(buf, "{}", i).unwrap();

            x.inspect(ui, &buf);
        }
    });
}

/// Inspects a sequence of nested inspectable items
pub fn nest<R, F: FnOnce() -> R>(ui: &Ui, label: &str, closure: F) -> Option<R> {
    imgui::TreeNode::new(label)
        // .opened(true, imgui::Condition::FirstUseEver)
        .flags(imgui::TreeNodeFlags::OPEN_ON_ARROW | imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK)
        .build(ui, closure)
}

/// Implements `Inspect` for an iterable type
#[macro_export]
macro_rules! impl_unit_enum {
    ($ty:ident $(, $var:ident)* $(,)?) => {
        impl $crate::Inspect for $ty {
            fn inspect(&mut self, ui: &$crate::imgui::Ui, label: &str) {
                $crate::unit_enum! {
                    ui, label, $($var),*
                };
            }
        }
    };
}

/// Inspects a unit enum
#[macro_export]
macro_rules! unit_enum {
    ($ui:expr, $label:expr, $E:ty $(, $var:ident)* $(,)?) => {{
        const VARIANTS: &[$E] = &[
            $(
                <$E>::$var,
            )*
        ];

        const VARIANT_NAMES: &[&'static str] = &[
            $(
                $crate::imgui::im_str!(
                    "{}",
                    stringify!(<$E>::$var),
                ),
            )*
        ];

        fn item_ix(variant: &$E) -> Option<usize> {
            VARIANTS
                .iter()
                .enumerate()
                .find_map(|(i, v)| if v == variant { Some(i) } else { None })
        }

        let flags = $crate::imgui::TreeNodeFlags::OPEN_ON_ARROW |
            $crate::imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK;

        if let Some(_) = $crate::imgui::TreeNode::new(label)
            .flags(flags)
            .build(ui, || {
                let mut buf = String::new();

                for (ix, v) in VARIANTS.iter().enumerate() {
                    if $crate::imgui::ComboBox::new($label)
                        .build_simple(
                            $ui,
                            &mut ix,
                            VARIANTS,
                            &|v| {
                                let i = item_ix(v).unwrap();
                                std::borrow::Cow::Borrowed(VARIANT_NAMES[i])
                            },
                        )
                    {
                        *self = VARIANTS[ix].clone();
                    }
                }
            });
    }};
}
