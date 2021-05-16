use std::{
    iter,
    thread, 
    time::Duration
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window}
};

struct State {
    surface : wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
}

impl State {

    // async keyword transforms block of 
    // code into a state machine
    // that implements the `Future` trait.
    // Normally callinga Blocking function would block the whole thread
    // blocked `Future`s will yield control of the thread to other `Future`s

    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // `instance` is a handle to the GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        // `surface` is used to create swapchain
        // for more on swapchains: [swap chain](https://en.wikipedia.org/wiki/Swap_chain)

        let surface = unsafe { instance.create_surface(window) };

        // `adapter` is needed to create the device and queue
        // `features` field in DeviceDescriptor allows us to specify extra features

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                },
            ).await.unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // todo!()
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { 
                label: Some("Render Encoder"),
            });

        // `encoder.begin_render_pass()` borrows `encoder` mutably 
        // therefore, `encoder.finish()` cannot be called
        // until the mutable borrow is released by `encoder.begin_render_pass()

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            
            // RenderPassDescriptor has three fields: `label`, `color_attachment`, and `depth_stencil_attachment`
            // color_attachments describes where color will be drawn to

            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                
                // `attachment` informs wgpu what textures to save the colors to
                // in this case, we have specified frame.view 
                // (that was created with swap_chain.get_current_frame())
                // esentially any colors drawn to this attachment will be drawn to the screen
                
                attachment: &frame.view,
                
                // `resolve_target` is the texture that will receive the resolved output
                // This will be the same as `attachment` unless multisampling is enabled

                resolve_target: None,

                // `ops` takes a `wgpu::Operations` object. this tells wgpu
                // what to do with the colors on the screen (specified by frame.view) 
                // `load` tells wgpu how to handle colors stored from the previous frame

                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.3,
                        b: 0.4,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        // release the mutable borrow
        // so that `finish` may be called by encoder.
        drop(render_pass);
        self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}

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
    let mut state: State = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
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
