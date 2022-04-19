mod camera;
mod input;
mod renderer;
mod texture;
mod vertex;
mod light;
mod quad;
mod buffers;

use renderer::Renderer;
use winit::{
    dpi::{PhysicalSize, Size, Position, PhysicalPosition},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(Size::Physical(PhysicalSize::<u32>::new(800, 600)))
        .with_position(Position::Physical(PhysicalPosition::<i32>::new((2560-800)/2 , (1440-600)/2)))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let mut state = pollster::block_on(Renderer::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            state.update();
            state.input().frame();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.get_size()),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => window.request_redraw(),
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            &WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: key_state,
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => state.input().update_key(key, key_state),
            &WindowEvent::MouseWheel { delta, .. } => state.input().update_wheel(delta),
            &WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => state.input().update_button(button, button_state),
            &WindowEvent::CursorMoved { position, .. } => {
                state.input().update_cursor(position)
            }
            WindowEvent::Resized(size) => state.resize(*size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size)
            }
            _ => {}
        },
        _ => {}
    });
}
