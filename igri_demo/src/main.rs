/*!
`igri` demo

Don't expect a good getting started tutorial of windowing libraries, graphics API and ImGUI!
*/

// #![feature(trace_macros)]
// trace_macros!(true);

use anyhow::*;

use glutin::{
    dpi, event_loop::EventLoop, platform::macos::WindowBuilderExtMacOS, window::WindowBuilder,
};

use igri::Inspect;

use igri_demo::ContextWrapper;

#[derive(Debug, Clone, PartialEq, Inspect)]
#[inspect(with = "inspect_newtype")]
pub struct NewType(u32);

fn inspect_newtype(x: &mut NewType, ui: &imgui::Ui, label: &str) {
    x.0.inspect(ui, label);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Uninspectable(String);

fn inspect_uninspectable(x: &mut Uninspectable, ui: &imgui::Ui, label: &str) {
    x.0.inspect(ui, "<manual inspect for non-Inspect>");
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub enum AttrDemoEnum {
    Named {
        x: f32,
        #[inspect(skip)]
        y: f32,
        // #[with = "inspect_f32"]
        z: f32,
    },
    Tuple(u32, #[inspect(skip)] f32, String),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct AttrDemo {
    #[inspect(skip)]
    hidden: u32,
    #[inspect(with = "inspect_f32")]
    manual: f32,
    newtype: NewType,
    #[inspect(with = "inspect_uninspectable")]
    uninspectable: Uninspectable,
    enums: Vec<AttrDemoEnum>,
}

fn inspect_f32(x: &mut f32, ui: &imgui::Ui, _label: &str) {
    x.inspect(ui, "<manual inspect>");
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Entity {
    pub name: String,
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
    pub ty: EntityType,
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub enum EntityType {
    Player { items: Vec<f32> },
    Enemy { ai: usize, item: Option<f32> },
    Unknown,
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new();

    let context_wrapper: ContextWrapper = {
        let window_builder = WindowBuilder::new()
            .with_title("igri demo")
            .with_inner_size(dpi::LogicalSize::new(1280, 720))
            // Don't use high DPI. Or I don't know what's happening 😬
            .with_disallow_hidpi(true)
            // Don't resize. Or I don't know what's happening 😬
            .with_resizable(false);

        glutin::ContextBuilder::new()
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
            .with_gl_debug_flag(true)
            .build_windowed(window_builder, &event_loop)?
    };

    let mut entities = vec![
        Entity {
            name: "Player".to_string(),
            hp: 100,
            atk: 50,
            def: 40,
            ty: EntityType::Player {
                items: vec![0.0, 100.0, 84.0],
            },
        },
        Entity {
            name: "Looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong name enemy".to_string(),
            hp: 2000,
            atk: 500000,
            def: 4,
            ty: EntityType::Enemy {
                ai: 100,
                item: None,
            },
        },
        Entity {
            name: r#"Unknown enemy. Multiline
text is here.

Ooooooooh
"#
            .to_string(),
            hp: 10,
            atk: 0,
            def: 300000,
            ty: EntityType::Unknown,
        },
    ];

    let mut demo = AttrDemo {
        hidden: 100,
        manual: 200.0,
        newtype: NewType(100),
        uninspectable: Uninspectable("Uninspectable".to_string()),
        enums: vec![
            AttrDemoEnum::Named {
                x: 0.0,
                y: 10.0,
                z: 100.0,
            },
            AttrDemoEnum::Tuple(0, 10.0, "tuple".to_string()),
            AttrDemoEnum::Unit,
        ],
    };

    igri_demo::run(event_loop, context_wrapper, move |ui| {
        ui.show_demo_window(&mut true);

        imgui::Window::new("Runtime inspector")
            .size([800.0, 600.0], imgui::Condition::FirstUseEver)
            // semi-transparent window
            .bg_alpha(0.5)
            .build(ui, || {
                demo.inspect(ui, "attribute demo");
                entities.inspect(ui, "entities");
            });
    })
}
