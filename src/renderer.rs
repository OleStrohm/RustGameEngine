use crate::buffers::{InstanceBuffer, Storage, Uniform};
use crate::camera::{Camera, CameraController};
use crate::input::{InputHandler, Key};
use crate::light::Light;
use crate::quad::{DrawQuad, Instance, InstanceRaw, Quad};
use crate::texture::{self, DepthTexture, Texture};
use crate::vertex::Vertex;
use cgmath::Rotation3;
use std::borrow::Cow;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Renderer {
    context: Context,
    input_handler: InputHandler,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: DepthTexture,
    quad: Quad,
    diffuse_texture: Texture,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: Uniform<Camera>,
    instances: Vec<Instance>,
    instance_buffer: InstanceBuffer<Instance>,
    instances_to_draw: usize,
    lights_uniform: Uniform<Light>,
    lights_storage: Storage<Light>,
    lights: Vec<Light>,
    light_render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let context = Context::new(window).await;

        let device = &context.device;
        let config = &context.config;

        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_texture = Texture::from_bytes(&context, diffuse_bytes, "happy tree").unwrap();

        let camera = Camera::basic(config.width, config.height, cgmath::Deg(45.0));
        let camera_uniform = Uniform::new(&context, &camera);

        let light1 = Light::new([2.0, 2.0, -0.1], [1.0, 1.0, 1.0]);
        let light2 = Light::new([-2.0, -2.0, -0.1], [1.0, 1.0, 1.0]);
        let lights_uniform = Uniform::new(&context, &light1);
        let lights = vec![light1, light2];

        let lights_storage = Storage::new(&context, &lights);
        let _num_lights_uniform = Uniform::new(&context, 0u32);

        let render_pipeline = context.create_render_pipeline(
            include_str!("shader.wgsl").into(),
            &[
                &diffuse_texture.layout(),
                &camera_uniform.layout(),
                &lights_storage.layout(),
            ],
            &[Vertex::desc(), InstanceRaw::desc()],
            Some(texture::DepthTexture::DEPTH_FORMAT),
        );

        let light_render_pipeline = context.create_render_pipeline(
            include_str!("light.wgsl").into(),
            &[&camera_uniform.layout(), &lights_storage.layout()],
            &[Vertex::desc()],
            Some(texture::DepthTexture::DEPTH_FORMAT),
        );

        let depth_texture = DepthTexture::create_depth_texture(&device, &config);

        let quad = Quad::new(&device);

        let camera_controller = CameraController::new(0.2);

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|y| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| Instance {
                    position: cgmath::Vector3 {
                        x: x as f32,
                        y: y as f32,
                        z: 0.0,
                    } - INSTANCE_DISPLACEMENT,
                })
            })
            .collect::<Vec<_>>();

        let instance_buffer = InstanceBuffer::new(&context, &instances);

        let input_handler = InputHandler::new();

        Self {
            context,
            input_handler,
            render_pipeline,
            depth_texture,
            quad,
            diffuse_texture,
            camera,
            camera_controller,
            camera_uniform,
            instance_buffer,
            instances_to_draw: instances.len(),
            instances,
            lights_uniform,
            lights_storage,
            lights,
            light_render_pipeline,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.context.size = new_size;
            self.context.config.width = new_size.width;
            self.context.config.height = new_size.height;
            self.camera.resize(new_size.width, new_size.height);
            self.depth_texture =
                DepthTexture::create_depth_texture(&self.context.device, &self.context.config);
            self.context
                .surface
                .configure(&self.context.device, &self.context.config)
        }
    }

    pub fn update(&mut self) {
        if self.input().clicked(Key::Up) {
            self.instances_to_draw = (self.instances_to_draw + 1).clamp(0, self.instances.len());
        }
        if self.input().clicked(Key::Down) {
            self.instances_to_draw = self.instances_to_draw.saturating_sub(1);
        }
        if self.input().clicked(Key::Space) {
            self.instances = (0..NUM_INSTANCES_PER_ROW)
                .flat_map(|y| {
                    (0..NUM_INSTANCES_PER_ROW).map(move |x| Instance {
                        position: cgmath::Vector3 {
                            x: x as f32,
                            y: y as f32,
                            z: 0.0,
                        } - INSTANCE_DISPLACEMENT,
                    })
                })
                .filter(|_| rand::random())
                .collect::<Vec<_>>();
            self.instances_to_draw = self.instances_to_draw.clamp(0, self.instances.len() - 1);
        }

        self.camera_controller
            .update(&mut self.camera, &self.input_handler);

        let old_position: cgmath::Vector3<_> = self.lights[0].position.into();
        self.lights[0].position =
            (cgmath::Quaternion::from_axis_angle((0.0, 0.0, 1.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();
        //let old_position: cgmath::Vector3<_> = self.lights[1].position.into();
        //self.lights[1].position =
        //    (cgmath::Quaternion::from_axis_angle((0.0, 0.0, -1.0).into(), cgmath::Deg(1.0))
        //        * old_position)
        //        .into();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        let instances = &self.instances[..self.instances_to_draw];
        self.instance_buffer.update(&self.context, instances);
        self.camera_uniform.update(&self.context, &self.camera);
        self.lights_storage.update(&self.context, &self.lights);

        // Draw everything
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_texture.bind_group(), &[]);
        render_pass.set_bind_group(1, &self.camera_uniform.bind_group(), &[]);
        render_pass.set_bind_group(2, &self.lights_storage.bind_group(), &[]);

        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw_quad_indexed(&self.quad, 0..self.instances.len() as _);

        // Debug draw lights
        self.lights_uniform.update(&self.context, &self.lights[1]);
        render_pass.set_pipeline(&self.light_render_pipeline);
        render_pass.set_bind_group(0, &self.camera_uniform.bind_group(), &[]);
        render_pass.set_bind_group(1, &self.lights_storage.bind_group(), &[]);
        render_pass.draw_quad_indexed(&self.quad, 0..self.lights.len() as _);

        drop(render_pass);

        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn get_size(&self) -> PhysicalSize<u32> {
        self.context.size
    }

    pub fn input(&mut self) -> &mut InputHandler {
        &mut self.input_handler
    }
}

pub struct Context {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
}

impl Context {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    pub fn create_render_pipeline<'a>(
        &self,
        shader: Cow<'a, str>,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        vertex_layouts: &[wgpu::VertexBufferLayout],
        depth_format: Option<wgpu::TextureFormat>,
    ) -> wgpu::RenderPipeline {
        let shader = wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader),
        };

        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let shader = self.device.create_shader_module(&shader);

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: vertex_layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: self.config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
    }

    #[inline]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    #[inline]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

pub const NUM_INSTANCES_PER_ROW: u32 = 10;
pub const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5 - 0.5,
    NUM_INSTANCES_PER_ROW as f32 * 0.5 - 0.5,
    0.0,
);
