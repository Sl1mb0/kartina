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

use minimp3::Frame;
use std::iter;
use wgpu::util::DeviceExt;
use winit::window::Window;

mod camera;
mod vertex;

/// This structure is necessary to `stage`
/// the uniforms that correspond to the `camera` view.
struct UniformStaging {
    camera: camera::Camera,
    model_rotation: cgmath::Deg<f32>,
}

impl UniformStaging {
    fn new(camera: camera::Camera) -> Self {
        Self {
            camera,
            model_rotation: cgmath::Deg(0.0),
        }
    }

    /// update the uniforms with the necessary information
    /// so that the window will have the appropriate camera view.
    fn update_uniforms(&self, uniforms: &mut Uniforms) {
        uniforms.view_proj = (camera::OPENGL_TO_WGPU_MATRIX
            * self.camera.build_view_projection_matrix()
            * cgmath::Matrix4::from_angle_z(self.model_rotation))
        .into();
    }
}

/// Uniform data structure to keep track of the window view.
/// This structure is passed to the shader modules so that the
/// correct window viewing can be rendered.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    // cgmath cannot be used with bytemuck directly;
    // Matrix4 must be converted into 4x4 `[f32]`.
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

/// `State` keeps track of all of the information being passed to and from
/// The GPU, and is used to handle what gets drawn, where it gets drawn,
/// what data is sent to the GPU, etc.
///
/// More specifically, this object creates an interface with the graphics card
/// using the device and queue fields. These objects are then used to create buffers
/// whose format is specified when they are created so that the GPU can then read those
/// buffers and render the appropriate image.
pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    clear_color: wgpu::Color,
    uniforms: Uniforms,
    uniform_staging: UniformStaging,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    // async keyword transforms block of
    // code into a state machine
    // that implements the `Future` trait.
    // Normally calling a Blocking function would block the whole thread
    //` blocked `Future`s will yield control of the thread to other `Future`s

    /// Given a `Window` create a new `State` that
    /// manages what is drawn in the window.
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
            })
            .await
            .unwrap();
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
        let clear_color = wgpu::Color::WHITE;
        let vs_module =
            device.create_shader_module(&wgpu::include_spirv!("./shaders/shader.vert.spv"));
        let fs_module =
            device.create_shader_module(&wgpu::include_spirv!("./shaders/shader.frag.spv"));
        let camera = camera::Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let mut uniforms = Uniforms::new();
        let uniform_staging = UniformStaging::new(camera);
        uniform_staging.update_uniforms(&mut uniforms);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline!"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[vertex::Vertex::desc()],
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
                alpha_to_coverage_enabled: false,
            },
        });
        let vbo = vertex::Vertex::sphere_vertices(1.0);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vbo),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let ibo = vertex::Vertex::sphere_indices();
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&ibo),
            usage: wgpu::BufferUsage::INDEX,
        });
        let num_indices = ibo.len() as u32;
        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            clear_color,
            uniforms,
            uniform_staging,
            uniform_buffer,
            uniform_bind_group,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            size,
        }
    }

    /// Resize the window according to `new_size`.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    /// Uses a single decoded mp3 frame to generate a vertex buffer for a sphere
    /// whose vertices are colored according to the frame's data.
    pub fn input(&mut self, frame: &Frame) -> bool {
        let mut vertices = vertex::Vertex::sphere_vertices(1.0);
        for vertex in &mut vertices {
            let colors = [
                vertex.position[0] * frame.data[2] as f32 % 256.0,
                vertex.position[1] + frame.data[1] as f32 % 256.0,
                vertex.position[2] / frame.data[0] as f32 % 256.0,
            ];
            vertex.change_color(colors);
        }

        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsage::VERTEX,
            });
        true
    }

    /// Update the model so that it continually rotates.
    /// Uniform staging must be updated with model rotation,
    /// and the corresponding uniforms must be updated to reflect the model's rotation.
    /// The GPU then reads the new uniform buffer and renders the sphere accordingly.
    pub fn update(&mut self) {
        self.uniform_staging.model_rotation += cgmath::Deg(2.0);
        self.uniform_staging.update_uniforms(&mut self.uniforms);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    /// Render the image in the window according to the vertex and index buffers.
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
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
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
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
