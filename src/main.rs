/*    
Kartina is a GPU shader that renders a sphere colored using decoded mp3 frame data.
Copyright (C) 2021 Timothy Maloney

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use minimp3::{
    Decoder,
    Error,
};
use std::{
    thread, 
    fs::File,
    time::Duration,
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};

mod state;

/// todo!()
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
    let mut decoder = Decoder::new(File::open("./song/Can I Take A Picture With You.mp3").unwrap());

    use futures::executor::block_on;

    // main cannot be asynchronous,
    // so we need to block thread to create state
    let mut state: state::State = block_on(state::State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match decoder.next_frame() {
            Ok(frame) => { state.input(&frame); },
            // The song is over, let's close the window
            Err(Error::Eof) => { *control_flow = ControlFlow::Exit },
            Err(e) => panic!("{:?}", e),
        }
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                 match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {},
                    },
                    WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        },
                    WindowEvent::ScaleFactorChanged {new_inner_size, .. } => {
                        // `new_inner_size` is &&mut so it must be dereferenced twice
                        state.resize(**new_inner_size);
                    }
                    _ => {},
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
