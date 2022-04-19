use std::marker::PhantomData;

use bytemuck::{NoUninit, Pod};
use wgpu::{util::DeviceExt, Device};
use wgpu::{BindGroup, BindGroupLayout, BufferUsages};

use crate::renderer::{self, Context};
const VERTEX: BufferUsages = wgpu::BufferUsages::VERTEX;
const INDEX: BufferUsages = wgpu::BufferUsages::INDEX;
const UNIFORM: BufferUsages = wgpu::BufferUsages::UNIFORM;
const STORAGE: BufferUsages = wgpu::BufferUsages::STORAGE;
const COPY_DST: BufferUsages = wgpu::BufferUsages::COPY_DST;

pub struct Uniform<C> {
    uniform: Buffer,
    content_marker: PhantomData<C>,
}

impl<C: Pod> Uniform<C> {
    #[inline]
    pub fn new(context: &Context, content: impl Into<C>) -> Self {
        let raw = uniform(context.device(), &[content.into()]);
        let (bind_group, bind_group_layout) = create_uniform_bind_group(&context.device(), 0, &raw);

        Self {
            uniform: Buffer::new(raw, bind_group, bind_group_layout),
            content_marker: PhantomData,
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
    pub fn update(&self, context: &renderer::Context, content: impl Into<C>) {
        context.queue().write_buffer(
            &self.uniform.raw,
            0,
            bytemuck::cast_slice(&[content.into()]),
        );
    }
}

pub struct Storage<C> {
    storage: Buffer,
    content_marker: PhantomData<C>,
}

impl<C: Pod> Storage<C> {
    #[inline]
    pub fn new<T>(context: &Context, content: &[T]) -> Self
    where
        for<'a> &'a T: Into<C>,
    {
        let content = content.iter().map(|l| l.into()).collect::<Vec<C>>();
        let raw = storage(context.device(), content.as_ref());
        let (bind_group, bind_group_layout) = create_storage_bind_group(&context.device(), 0, &raw);

        Self {
            storage: Buffer::new(raw, bind_group, bind_group_layout),
            content_marker: PhantomData,
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn raw(&self) -> &Buffer {
        &self.storage
    }

    #[inline]
    pub fn layout(&self) -> &BindGroupLayout {
        &self.storage.layout()
    }

    #[inline]
    pub fn bind_group(&self) -> &BindGroup {
        &self.storage.bind_group()
    }

    #[inline]
    pub fn update<T>(&self, context: &renderer::Context, content: &[T])
    where
        for<'a> &'a T: Into<C>,
    {
        let content = content.iter().map(|l| l.into()).collect::<Vec<C>>();
        context
            .queue()
            .write_buffer(&self.storage.raw, 0, bytemuck::cast_slice(&content));
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

pub fn storage(device: &Device, contents: &[impl NoUninit]) -> wgpu::Buffer {
    buffer(device, "Uniform Buffer", contents, STORAGE | COPY_DST)
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

pub fn create_storage_bind_group(
    device: &Device,
    binding: u32,
    buffer: &wgpu::Buffer,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
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
