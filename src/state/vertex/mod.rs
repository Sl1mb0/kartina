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

/// structure to store `Vertex` information. Use of #[repr(C)]
/// so that the data structure can be read as a buffer of bytes by the GPU.
/// both bytemuck::* features are used so that bytemuck::cast_slice() can be used to
/// convert a slice of any type into a slice of `&[u8]`;
/// since `wgpu::BufferInitDescriptor { contents: &[u8], .. }` and the `wgpu::BufferInitDescriptor`
/// is what's used to create a buffer than can be read by the GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    /// change color value of a given vertex.
    ///
    /// # Examples
    /// ```
    /// let mut v = Vertex{ [1.0, 2.0, 3.0], [0.5, 0.5, 0.5] };
    /// v.change_color([2.0, 3.0, 5.0]);
    /// assert!(v.color == [2.0, 3.0, 5.0]);
    /// ```
    pub fn change_color(&mut self, new_color: [f32; 3]) -> &Self {
        for (index, value) in new_color.iter().enumerate() {
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
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }

    /// Returns a vector of vertices that correspond to a sphere
    /// of radius `r`, centered at the origin.
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
    /// This function specifically generates a sphere centered at the origin
    /// with radius `r`. The sphere consists of 18 stacks, and 36 total sectors.
    /// For more information, reference `OpenGL Sphere` in the references section of the `README.md`
    ///
    /// # Examples
    /// ```
    /// let mut vbo = sphere_vertices(4.0);
    /// vbo[3].change_color([1.0,2.0,3.0])
    /// assert!(vbo[3].color == [1.0,2.0,3.0]);
    /// ```
    pub fn sphere_vertices(r: f32) -> Vec<Vertex> {
        // vector to contain all vertices which will be returned
        let mut vertices = Vec::new();

        // vertex position
        let (mut x, mut y, mut z, mut xy): (f32, f32, f32, f32);
        let (mut stack_angle, mut sector_angle): (f32, f32);
        let (stack_step, sector_step): (f32, f32) = (PI / 18.0, 2.0 * PI / 36.0);

        for i in 0..=18 {
            stack_angle = PI / 2.0 - i as f32 * stack_step;

            xy = (r / 10.0) * stack_angle.cos();
            z = (r / 10.0) * stack_angle.sin();
            for j in 0..=36 {
                sector_angle = j as f32 * sector_step;

                x = xy * sector_angle.cos();
                y = xy * sector_angle.sin();
                vertices.push(Vertex {
                    position: [x, y, z],
                    color: [0.0, 0.0, 0.0],
                });
            }
        }
        vertices
    }

    /// Generate an array of indices who correspond to an individual vertex in a vertex buffer.
    /// Each thruple of indices in the array corresponds to  triangle that is a individual component
    /// of a larger image.
    ///
    /// The order in which are connected to form an individual triangle in GLSL is counter clock-wise
    /// as an example, consider this triangle, that is connected by the vertices top, left, right.
    /// Their vertex array is: [top,left,right],
    /// so the index array would then look like: [0,1,2]
    /// The vertices are then connected using the index buffer in a counter-clockwise order:
    ///
    ///             top
    ///             /\
    ///            /  \
    ///           /    \
    ///      left/______\right
    ///
    /// # Examples
    /// ```
    /// let ibo = sphere_indices();
    /// ```
    pub fn sphere_indices() -> Vec<u32> {
        let mut indices = Vec::new();

        for i in 0..18 {
            let mut k1 = i * 37; // # of sectors + 1
            let mut k2 = k1 + 37;
            for _j in 0..36 {
                if i != 0 {
                    indices.push(k1);
                    indices.push(k2);
                    indices.push(k1 + 1);
                }
                if i != 17 {
                    // # of stacks - 1
                    indices.push(k1 + 1);
                    indices.push(k2);
                    indices.push(k2 + 1);
                }
                k1 += 1;
                k2 += 1;
            }
        }
        indices
    }
}

#[cfg(test)]
#[test]
fn test_change_color() {
    let position = [1.0, 2.0, 3.0];
    let color = position;
    let mut v = Vertex { position, color };
    v.change_color([4.0, 7.5, 9.0]);
    assert!([4.0, 7.5, 9.0] == v.color);
}
#[test]
fn test_vertices() {
    let vertices = Vertex::sphere_vertices(1.0);
    let mut test = Vec::new();
    let (mut x, mut y, mut z, mut xy): (f32, f32, f32, f32);
    let (mut stack_angle, mut sector_angle): (f32, f32);
    let (stack_step, sector_step): (f32, f32) = (PI / 18.0, 2.0 * PI / 36.0);

    for i in 0..=18 {
        stack_angle = PI / 2.0 - i as f32 * stack_step;

        xy = 0.1 * stack_angle.cos();
        z = 0.1 * stack_angle.sin();
        for j in 0..=36 {
            sector_angle = j as f32 * sector_step;

            x = xy * sector_angle.cos();
            y = xy * sector_angle.sin();
            test.push(Vertex {
                position: [x, y, z],
                color: [0.0, 0.0, 0.0],
            });
        }
    }
    for (index, vertex) in vertices.iter().enumerate() {
        assert!(vertex.color == test[index].color);
        assert!(vertex.position == test[index].position);
    }
}
#[test]
fn test_indices() {
    let indices = Vertex::sphere_indices();
    let mut test = Vec::new();

    for i in 0..18 {
        let mut k1 = i * 37;
        let mut k2 = k1 + 37;
        for _j in 0..36 {
            if i != 0 {
                test.push(k1);
                test.push(k2);
                test.push(k1 + 1);
            }
            if i != 17 {
                test.push(k1 + 1);
                test.push(k2);
                test.push(k2 + 1);
            }
            k1 += 1;
            k2 += 1;
        }
    }
    assert!(test == indices);
}
