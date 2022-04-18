use std::ops::Range;

use crate::buffers;
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
