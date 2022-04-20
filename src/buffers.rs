use std::borrow::Borrow;
use std::marker::PhantomData;
use std::ops::RangeBounds;

use bytemuck::{NoUninit, Pod};
use wgpu::{util::DeviceExt, Device};
use wgpu::{BindGroup, BindGroupLayout, BufferAddress, BufferUsages};

use crate::renderer::{self, Context};
const VERTEX: BufferUsages = wgpu::BufferUsages::VERTEX;
const INDEX: BufferUsages = wgpu::BufferUsages::INDEX;
const UNIFORM: BufferUsages = wgpu::BufferUsages::UNIFORM;
const STORAGE: BufferUsages = wgpu::BufferUsages::STORAGE;
const COPY_DST: BufferUsages = wgpu::BufferUsages::COPY_DST;

pub trait ToData {
    type Data: Pod;

    fn to_data(&self) -> Self::Data;
}

impl ToData for u32 {
    type Data = u32;

    fn to_data(&self) -> Self::Data {
        *self
    }
}

pub struct Uniform<C> {
    uniform: Buffer<C>,
}

impl<C: ToData> Uniform<C> {
    #[inline]
    pub fn new(context: &Context, content: impl Borrow<C>) -> Self {
        let raw = uniform(context.device(), &[content.borrow().to_data()]);
        let (bind_group, bind_group_layout) = create_uniform_bind_group(&context.device(), 0, &raw);

        Self {
            uniform: Buffer::new(raw, bind_group, bind_group_layout),
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn raw(&self) -> &Buffer<C> {
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
    pub fn update(&self, context: &renderer::Context, content: impl Borrow<C>) {
        context.queue().write_buffer(
            &self.uniform.raw,
            0,
            bytemuck::cast_slice(&[content.borrow().to_data()]),
        );
    }
}

pub struct Storage<C> {
    storage: Buffer<C>,
    length_buffer: wgpu::Buffer,
}

impl<C: ToData> Storage<C> {
    #[inline]
    pub fn new(context: &Context, content: impl AsRef<[C]>) -> Self {
        let content = content
            .as_ref()
            .iter()
            .map(ToData::to_data)
            .collect::<Vec<C::Data>>();
        let storage_buffer = storage(context.device(), content.as_ref());
        let length_buffer = uniform(context.device(), &[0u32]);
        let (bind_group, bind_group_layout) =
            create_storage_bind_group(&context.device(), &storage_buffer, &length_buffer);

        Self {
            storage: Buffer::new(storage_buffer, bind_group, bind_group_layout),
            length_buffer,
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn raw(&self) -> &Buffer<C> {
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
    pub fn update(&self, context: &Context, content: impl AsRef<[C]>) {
        let content = content
            .as_ref()
            .iter()
            .map(ToData::to_data)
            .collect::<Vec<C::Data>>();
        context
            .queue()
            .write_buffer(&self.storage.raw, 0, bytemuck::cast_slice(&content));
        context.queue().write_buffer(
            &self.length_buffer,
            0,
            bytemuck::cast_slice(&[content.len() as u32]),
        );
    }
}

pub struct InstanceBuffer<C> {
    raw: wgpu::Buffer,
    _content_marker: PhantomData<C>,
}

impl<C: ToData> InstanceBuffer<C> {
    #[inline]
    pub fn new(context: &Context, content: impl AsRef<[C]>) -> Self {
        let content = content
            .as_ref()
            .iter()
            .map(ToData::to_data)
            .collect::<Vec<C::Data>>();
        let raw = instance(context.device(), content.as_ref());

        Self {
            raw,
            _content_marker: PhantomData,
        }
    }

    #[inline]
    pub fn update(&self, context: &renderer::Context, content: impl AsRef<[C]>) {
        let content = content
            .as_ref()
            .iter()
            .map(ToData::to_data)
            .collect::<Vec<C::Data>>();
        context
            .queue()
            .write_buffer(&self.raw, 0, bytemuck::cast_slice(&content));
    }

    #[inline]
    pub fn slice<'a>(&'a self, bounds: impl RangeBounds<BufferAddress>) -> wgpu::BufferSlice<'a> {
        self.raw.slice(bounds)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.raw
    }
}

#[derive(Debug)]
pub struct Buffer<C> {
    raw: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    _content_marker: PhantomData<C>,
}

impl<C: ToData> Buffer<C> {
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
            _content_marker: PhantomData,
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    #[inline]
    pub fn update(&self, context: &renderer::Context, content: impl AsRef<[C]>) {
        let content = content
            .as_ref()
            .iter()
            .map(|l| l.to_data())
            .collect::<Vec<C::Data>>();
        context
            .queue()
            .write_buffer(&self.raw, 0, bytemuck::cast_slice(&content));
    }
}

pub fn uniform(device: &Device, contents: &[impl NoUninit]) -> wgpu::Buffer {
    buffer(device, "Uniform Buffer", contents, UNIFORM | COPY_DST)
}

pub fn storage(device: &Device, contents: &[impl NoUninit]) -> wgpu::Buffer {
    buffer(device, "Storage Buffer", contents, STORAGE | COPY_DST)
}

pub fn vertex(device: &wgpu::Device, contents: &[impl NoUninit]) -> wgpu::Buffer {
    buffer(device, "Vertex Buffer", contents, VERTEX)
}

pub fn instance(device: &wgpu::Device, contents: &[impl NoUninit]) -> wgpu::Buffer {
    buffer(device, "Instance Buffer", contents, VERTEX | COPY_DST)
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
    buffer: &wgpu::Buffer,
    length: &wgpu::Buffer,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: None,
    });

    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: length.as_entire_binding(),
            },
        ],
        label: None,
    });

    (group, layout)
}
