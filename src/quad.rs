use std::ops::Range;

use crate::buffers::{self, ToData};
use crate::vertex::Vertex;

pub const VERTICES: &[Vertex] = &[
    Vertex::new([-0.5, 0.5, 0.0], [0.0, 0.0]),
    Vertex::new([-0.5, -0.5, 0.0], [0.0, 1.0]),
    Vertex::new([0.5, -0.5, 0.0], [1.0, 1.0]),
    Vertex::new([0.5, 0.5, 0.0], [1.0, 0.0]),
];

pub const INDICES: &[u16] = &[0, 1, 3, 1, 2, 3];

pub struct Quad {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    num_indices: u32,
}

impl Quad {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices = buffers::vertex(device, VERTICES);
        let indices = buffers::index(device, INDICES);
        let num_indices = INDICES.len() as u32;

        Self {
            vertices,
            indices,
            num_indices,
        }
    }
}

pub trait DrawQuad<'quad> {
    fn draw_quad(&mut self, quad: &'quad Quad);
    fn draw_quad_indexed(&mut self, quad: &'quad Quad, instances: Range<u32>);
}

impl<'pass, 'quad> DrawQuad<'quad> for wgpu::RenderPass<'pass>
where
    'quad: 'pass,
{
    fn draw_quad(&mut self, quad: &'quad Quad) {
        self.draw_quad_indexed(quad, 0..1);
    }

    fn draw_quad_indexed(&mut self, quad: &'quad Quad, instances: Range<u32>) {
        self.set_vertex_buffer(0, quad.vertices.slice(..));
        self.set_index_buffer(quad.indices.slice(..), wgpu::IndexFormat::Uint16);
        self.draw_indexed(0..quad.num_indices, 0, instances);
    }
}

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl ToData for Instance {
    type Data = InstanceRaw;

    fn to_data(&self) -> Self::Data {
        Self::Data {
            model: cgmath::Matrix4::from_translation(self.position).into(),
        }
    }
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
