use std::iter::repeat;

use cgmath::Vector3;

#[derive(Debug)]
pub struct Light {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
}

impl Light {
    pub fn new(position: impl Into<Vector3<f32>>, color: impl Into<Vector3<f32>>) -> Self {
        Self {
            position: position.into(),
            color: color.into(),
        }
    }
}

impl From<&Light> for LightUniform {
    fn from(light: &Light) -> Self {
        Self {
            position: light.position.into(),
            _padding_pos: 0,
            color: light.color.into(),
            _padding_col: 0,
        }
    }
}

impl<V: AsRef<[Light]>> From<V> for LightsUniform {
    fn from(lights: V) -> Self {
        let lights = lights.as_ref();
        assert!(lights.len() <= 128);

        LightsUniform(
            lights
                .into_iter()
                .map(|light| LightUniform {
                    position: light.position.into(),
                    _padding_pos: 0,
                    color: light.color.into(),
                    _padding_col: 0,
                })
                .chain(repeat(Default::default()))
                .take(128)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct LightUniform {
    pub position: [f32; 3],
    _padding_pos: u32,
    pub color: [f32; 3],
    _padding_col: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightsUniform([LightUniform; 128]);
