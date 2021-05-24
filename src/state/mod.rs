use std::iter;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    window::Window
};

mod vertex;

pub struct State {
    surface : wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    pub size: winit::dpi::PhysicalSize<u32>
}

impl State {

    // async keyword transforms block of 
    // code into a state machine
    // that implements the `Future` trait.
    // Normally calling a Blocking function would block the whole thread
    // blocked `Future`s will yield control of the thread to other `Future`s

    pub async fn new(window: &Window) -> Self {
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

        let clear_color = wgpu::Color::BLACK;

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));

        let render_pipeline_layout = 
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[]
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline!"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[vertex::Vertex::desc()]
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                // setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertex::VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(vertex::INDICES),
                usage: wgpu::BufferUsage::INDEX,
            }
        );

        let num_indices = vertex::INDICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            clear_color,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            size
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    // TODO: change argument from event: &WindowEvent to decoded_mp3_data: &i16
    // TODO: use decoded_mp3_data to change colors, draw shapes, etc.
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0
                };
                true
            },
            WindowEvent::KeyboardInput { input: KeyboardInput {
                state,
                virtual_keycode: Some(VirtualKeyCode::Space),
                ..
                }, 
            ..
            } => { 
                    if *state != ElementState::Released {
                        self.clear_color = wgpu::Color {
                            r: self.clear_color.r * self.size.width as f64, 
                            g: self.clear_color.g * self.size.height as f64,
                            b: 1.0,
                            a: 1.0
                        };
                    }
                    true 
            },
            _ => false
        }
    }

    pub fn update(&mut self) {
        // todo!()
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
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

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            
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
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: true
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        // release the mutable borrow
        // so that `finish` may be called by encoder.

        drop(render_pass);
        self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}
