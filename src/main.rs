use std::{
    thread, 
    time::Duration
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder}
};

mod state;

fn main() {
    thread::spawn(|| {
        // child thread
        play::play("./song/Can I Take A Picture With You.mp3").unwrap();
        thread::sleep(Duration::from_millis(1));
    });
    // parent thread
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    use futures::executor::block_on;

    // main cannot be asynchronous,
    // so we need to block thread to create state

    let mut state: state::State = block_on(state::State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                // TODO: change state.input() argument to take vec<i16>;
                // TODO: use minimp3 decoder to read decoded mp3 data 
                // TODO: pass decoded mp3 data (16-bit signed integer) to state.input()
                /* 

                */
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged {new_inner_size, .. } => {
                            // `new_inner_size` is &&mut so it must be dereferenced twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {},
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {

                // RedrawRequested will only trigger once
                // unless it is manually requested

                window.request_redraw();
            }
            _ => {}
        }
    });
}
