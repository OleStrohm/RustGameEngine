use cgmath::Rad;

use crate::buffers::ToData;
use crate::input::InputHandler;
use crate::input::Key;

#[derive(Debug)]
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: Rad<f32>,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn basic(width: u32, height: u32, fovy: impl Into<Rad<f32>>) -> Camera {
        Camera::new(width as f32 / height as f32, fovy, 0.1, 100.0)
    }

    pub fn new(aspect: f32, fovy: impl Into<Rad<f32>>, znear: f32, zfar: f32) -> Camera {
        Camera {
            eye: (0.0, 0.0, 1.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let size = 5.0;
        let proj = cgmath::ortho(
            -size * self.aspect,
            size * self.aspect,
            -size,
            size,
            10.0,
            -10.0,
        );

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}

pub struct CameraController {
    speed: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }

    pub fn update(&self, camera: &mut Camera, input: &InputHandler) {
        let mut disp: cgmath::Vector3<f32> = (0.0, 0.0, 0.0).into();
        if input.down(Key::W) {
            disp += (0.0, self.speed, 0.0).into();
        }
        if input.down(Key::S) {
            disp -= (0.0, self.speed, 0.0).into();
        }

        if input.down(Key::D) {
            disp += (self.speed, 0.0, 0.0).into();
        }
        if input.down(Key::A) {
            disp -= (self.speed, 0.0, 0.0).into();
        }

        camera.eye += disp;
        camera.target += disp;
    }
}

impl ToData for Camera {
    type Data = CameraUniform;

    fn to_data(&self) -> Self::Data {
        CameraUniform {
            view_position: self.eye.to_homogeneous().into(),
            view_proj: (OPENGL_TO_WGPU_MATRIX * self.build_view_projection_matrix()).into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
