use std::ops::{Deref, DerefMut};

use bytemuck::{NoUninit, Pod};
use wgpu::{BufferUsages, BindGroupLayout, BindGroup};
use wgpu::{util::DeviceExt, Device};
const VERTEX: BufferUsages = wgpu::BufferUsages::VERTEX;
const INDEX: BufferUsages = wgpu::BufferUsages::INDEX;
const UNIFORM: BufferUsages = wgpu::BufferUsages::UNIFORM;
const COPY_DST: BufferUsages = wgpu::BufferUsages::COPY_DST;

pub struct Uniform<C> {
    uniform: Buffer,
    content: C,
}

impl<C: Pod> Uniform<C> {
    #[inline]
    pub fn new(device: &Device, content: C) -> Self {
        let raw = uniform(device, &[content]);
        let (bind_group, bind_group_layout) = create_uniform_bind_group(&device, 0, &raw);

        Self {
            uniform: Buffer::new(raw, bind_group, bind_group_layout),
            content,
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn raw(&self) -> &Buffer {
        &self.uniform
    }

    #[inline]
    pub fn layout(&self) -> &BindGroupLayout {
        &self.uniform.layout()
    }

    #[inline]
    pub fn bind_group(&self) -> &BindGroup {
        &self.uniform.bind_group()
    }

    #[inline]
    pub fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.uniform.raw, 0, bytemuck::cast_slice(&[self.content]));
    }
}

impl<C> Deref for Uniform<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<C> DerefMut for Uniform<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

#[derive(Debug)]
pub struct Buffer {
    raw: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Buffer {
    #[inline]
    pub fn new(
        raw: wgpu::Buffer,
        bind_group: wgpu::BindGroup,
        bind_group_layout: wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            raw,
            bind_group,
            bind_group_layout,
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.raw
    }

    #[inline]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    #[inline]
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

pub fn uniform(device: &Device, contents: &[impl NoUninit]) -> wgpu::Buffer {
    buffer(device, "Uniform Buffer", contents, UNIFORM | COPY_DST)
}

pub fn vertex(device: &wgpu::Device, contents: &[impl NoUninit]) -> wgpu::Buffer {
    buffer(device, "Vertex Buffer", contents, VERTEX)
}

pub fn instances(device: &wgpu::Device, contents: &[impl NoUninit]) -> wgpu::Buffer {
    buffer(device, "Instance Buffer", contents, VERTEX)
}

pub fn index(device: &wgpu::Device, contents: &[impl NoUninit]) -> wgpu::Buffer {
    buffer(device, "Index Buffer", contents, INDEX)
}

pub fn buffer(
    device: &wgpu::Device,
    label: &str,
    contents: &[impl NoUninit],
    usage: BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(contents),
        usage,
    })
}

pub fn create_uniform_bind_group(
    device: &Device,
    binding: u32,
    buffer: &wgpu::Buffer,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: None,
    });

    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding,
            resource: buffer.as_entire_binding(),
        }],
        label: None,
    });

    (group, layout)
}
