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

use std::f32::consts::PI;
                                                                                  
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3]
}

impl Vertex {
    /// change color value of a given vertex
    pub fn change_color(&mut self, new_color: [f32; 3]) -> &Self{ 
        for (index,value) in new_color.iter().enumerate() {
            self.color[index] = *value;
        }
        self
    }

    /// Return a description of the layout for the vertex buffer.
    /// More specifically, the vertex shader needs to know where in memory to 
    /// look for the vertex information, and how that information is organized,
    /// so that it may read and generate said vertices.
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

    /// Returns a vector of vertices that correspond to a sphere
    /// of radius `r` whose center is at `center`
    /// 
    /// The definition of a sphere is a 3d closed surface wher every point on 
    /// the sphere is the same distance (radius) from an arbitrary center point.
    /// 
    /// The equation of a sphere at the origin is x^2 + y^2 + z^2.
    /// A sphere is drawn by first, sampling a limited amount of points from the sphere.
    /// 
    /// The sphere itself is then divided up vertically and horizontally, creating
    /// a cross section of the sphere where the horizontal sections are composed of
    /// individual `sectors` and the vertical sections are composed of individual `stacks`.
    /// Together, the sectors and stacks compose the surface of the sphere.
    ///
    /// # Reference 
    /// [OpenGL Sphere](http://www.songho.ca/opengl/gl_sphere.html)
    pub fn sphere_vertices(r: f32) -> Vec<Vertex> {
        // vector to contain all vertices which will be returned
        let mut vertices = Vec::new();

        // vertex position
        let (mut x, mut y, mut z, mut xy): (f32,f32,f32,f32);
        let (mut stack_angle, mut sector_angle): (f32,f32);
        let (mut stack, mut sector): (f32,f32) = (18.0, 36.0);
        let (stack_step, sector_step): (f32,f32) = (PI/stack, 2.0*PI/sector);

        while stack > 0.0 {
            stack_angle = PI/2.0 - stack *stack_step;

            xy = r * stack_angle.cos();
            z = r * stack_angle.sin();
            while sector > 0.0 {
                sector_angle = sector * sector_step;

                x = xy * sector_angle.cos();
                y = xy * sector_angle.sin();
                vertices.push(Vertex{position: [x,y,z], color: [0.0,0.0,0.0]});
                sector -= 1.0;
            }
            stack -= 1.0;
        }
        vertices
    }
    

    pub fn sphere_indices() -> Vec<u32> {
        let mut indices = Vec::new();

        let (mut stack, mut sector) = (18, 36);
        while stack > 0 {
            let mut k1 = stack * 37; // sector + 1.0
            let mut k2 = k1 + 37;
            while sector > 0 {
                if stack != 0 {
                    indices.push(k1);
                    indices.push(k2);
                    indices.push(k1+1);
                }
                if stack != 17 {  // stack - 1.0
                    indices.push(k1+1);
                    indices.push(k2);
                    indices.push(k2+1);
                }
                k1 += 1;
                k2 += 1;
                sector -= 1;
            }
            stack -= 1;
        }
        indices
    }
}
