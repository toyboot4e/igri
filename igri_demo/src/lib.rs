use std::time::Instant;

use anyhow::*;

use glow::HasContext;
use glutin::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    WindowedContext,
};

use imgui_winit_support::WinitPlatform;

pub type Event<'a> = glutin::event::Event<'a, ()>;

pub type Window = WindowedContext<glutin::PossiblyCurrent>;
pub type ContextWrapper = glutin::WindowedContext<glutin::NotCurrent>;

pub fn run(
    event_loop: EventLoop<()>,
    context_wrapper: ContextWrapper,
    mut render: impl FnMut(&imgui::Ui) + 'static,
) -> Result<()> {
    let window = unsafe { context_wrapper.make_current().map_err(|_| anyhow!("omg"))? };
    let gl = { unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s)) } };

    let (mut winit_platform, mut imgui_context) = init_imgui(&window);

    let mut ig_renderer = imgui_glow_renderer::AutoRenderer::initialize(gl, &mut imgui_context)
        .expect("failed to create renderer");

    let mut last_frame = Instant::now();

    // Standard winit event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui_context
                    .io_mut()
                    .update_delta_time(now.duration_since(last_frame));
                last_frame = now;
            }
            Event::MainEventsCleared => {
                winit_platform
                    .prepare_frame(imgui_context.io_mut(), window.window())
                    .unwrap();
                window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                // The renderer assumes you'll be clearing the buffer yourself
                unsafe { ig_renderer.gl_context().clear(glow::COLOR_BUFFER_BIT) };

                let ui = imgui_context.frame();

                (render)(&ui);

                winit_platform.prepare_render(&ui, window.window());
                let draw_data = ui.render();

                // This is the only extra render step to add
                ig_renderer
                    .render(draw_data)
                    .expect("error rendering imgui");

                window.swap_buffers().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            event => {
                winit_platform.handle_event(imgui_context.io_mut(), window.window(), &event);
            }
        }
    });
}

fn init_imgui(window: &Window) -> (WinitPlatform, imgui::Context) {
    let mut imgui_context = imgui::Context::create();
    imgui_context.set_ini_filename(None);

    // TODO: change on platform
    // imgui_context.io_mut().display_framebuffer_scale = [1.0, 1.0];

    let mut winit_platform = WinitPlatform::init(&mut imgui_context);
    winit_platform.attach_window(
        imgui_context.io_mut(),
        window.window(),
        imgui_winit_support::HiDpiMode::Rounded,
    );

    imgui_context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    // imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;

    (winit_platform, imgui_context)
}
