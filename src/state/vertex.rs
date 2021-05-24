#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3]
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3
                }
            ]
        }
    }
}                                                                                 //              A
                                                                                  //
pub const VERTICES: &[Vertex] = &[                                                //             /\
    Vertex { position: [0.0868241, 0.49240386, 0.0], color: [1.0, 0.5, 0.0] },    // A          /  \
    Vertex { position: [-0.49513406, -0.258647, 0.0], color: [0.0, 1.0, 0.0] },   // B        B/____\E
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [1.0, 0.0, 1.0] }, // C         |  / |
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.7, 0.5, 0.0]  },  // D         | /  | 
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.2, 0.0, 0.0] },    // E        C|/___|D
    Vertex { position: [0.6983213, 0.70932157, 0.0], color: [0.2, 0.0, 0.0] },
];
/*
pub const CUBE: &[Vertex] = &[
    Vertex {}
    Vertex {}
    Vertex {}
    Vertex {}
    Vertex {}
    Vertex {}
    Vertex {}
    Vertex {}
]*/

pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
    4, 3, 5 
];
