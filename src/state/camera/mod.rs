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

/// this is a `camera` data structure to keep
/// track of all information related to the window's view.
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    /// `view` is an inverse of the camera's transform matrix.
    /// It moves the 'world' to be at the position and rotation of the camera.
    /// `proj` matrix wraps the scene to provide depth.
    ///
    /// Since the coordinate system in wgpu is based on DirectX and Metal's coordinate
    /// systems (-1.0 <= x,y <= 1.0 && 0.0 <= z <= 1.0) means that a transformation
    /// matrix is necessary to translate and scale the scene from OpenGl's coordinate
    /// system to WGPU's.
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }
}

/// transformation matrix for scaling OpenGL's coordinate system to WGPU's.
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

#[cfg(test)]
#[test]
fn test_view_projection_matrix() {
    let camera = Camera {
        eye: (0.0, 5.0, 4.0).into(),
        target: (0.0, 0.0, 0.0).into(),
        up: cgmath::Vector3::unit_y(),
        aspect: 5.0,
        fovy: 34.0,
        znear: 0.4,
        zfar: 98.0,
    };
    let view = cgmath::Matrix4::look_at_rh(
        (0.0, 5.0, 4.0).into(),
        (0.0, 0.0, 0.0).into(),
        cgmath::Vector3::unit_y(),
    );
    let proj = cgmath::perspective(cgmath::Deg(34.0), 5.0, 0.4, 98.0);
    let test: cgmath::Matrix4<f32> = proj * view;
    assert_eq!(test, camera.build_view_projection_matrix());
}
