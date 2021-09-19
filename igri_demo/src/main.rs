/*!
`igri` demo

Don't expect a good getting started tutorial of windowing libraries,
graphics API and ImGUI!
*/

use anyhow::*;

use glutin::{
    dpi, event_loop::EventLoop, platform::macos::WindowBuilderExtMacOS, window::WindowBuilder,
};

use igri::Inspect;

use igri_demo::ContextWrapper;

pub struct GameData {}

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
    Player { items: Vec<usize> },
    Enemy { ai: usize },
    Unknown,
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new();

    let context_wrapper: ContextWrapper = {
        let window_builder = WindowBuilder::new()
            .with_title("igri demo")
            .with_inner_size(dpi::LogicalSize::new(1280, 720))
            // Don't use high DPI. Or else, I don't know what's happening 😬
            .with_disallow_hidpi(true)
            // Don't resize. Or else, I don't know what's happening 😬
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
                items: vec![0, 1, 2, 3],
            },
        },
        Entity {
            name: "Entity1".to_string(),
            hp: 2000,
            atk: 500000,
            def: 4,
            ty: EntityType::Enemy { ai: 100 },
        },
        Entity {
            name: "Unknown1".to_string(),
            hp: 10,
            atk: 0,
            def: 300000,
            ty: EntityType::Unknown,
        },
    ];

    igri_demo::run(event_loop, context_wrapper, move |ui| {
        ui.show_demo_window(&mut true);

        imgui::Window::new("Runtime inspector")
            .size([600.0, 400.0], imgui::Condition::FirstUseEver)
            // semi-transparent window
            .bg_alpha(0.5)
            .build(ui, || {
                entities.inspect(ui, "entities");
            });
    })
}