/*!
`igri` demo

Don't expect a good getting started tutorial of windowing libraries,
graphics API and ImGUI!
*/

#![feature(trace_macros)]

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

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Entity {
    pub name: String,
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
    pub ty: EntityType,
}

trace_macros!(true);

#[derive(Debug, Clone, PartialEq, Inspect)]
pub enum EntityType {
    Player { items: Vec<f32> },
    Enemy { ai: usize, item: Option<f32> },
    Unknown,
}

trace_macros!(false);

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

    let mut wrapper = NewType(100);

    igri_demo::run(event_loop, context_wrapper, move |ui| {
        ui.show_demo_window(&mut true);

        imgui::Window::new("Runtime inspector")
            .size([800.0, 600.0], imgui::Condition::FirstUseEver)
            // semi-transparent window
            .bg_alpha(0.5)
            .build(ui, || {
                wrapper.inspect(ui, "new type wrapper");
                entities.inspect(ui, "entities");
            });
    })
}
