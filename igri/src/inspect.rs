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

/// Standard method to nest a tree node
pub fn nest<R, F: FnOnce() -> R>(ui: &Ui, label: &str, closure: F) -> Option<R> {
    imgui::TreeNode::new(label)
        // .opened(true, imgui::Condition::FirstUseEver)
        .flags(imgui::TreeNodeFlags::OPEN_ON_ARROW | imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK)
        .build(ui, closure)
}

