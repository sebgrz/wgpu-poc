use std::{borrow::Cow, sync::Arc};

use pollster::FutureExt as _;
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayoutEntry, BufferUsages, Device, Extent3d, Instance,
    InstanceDescriptor, Queue, RenderPipeline, RequestAdapterOptions, SamplerBindingType,
    SamplerDescriptor, ShaderStages, Surface, SurfaceConfiguration, TexelCopyTextureInfo,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

const SPRITES: &[u8] = include_bytes!("../spelunky_shop.png");
const OBJECTS: [Vec4f; 2] = [
    Vec4f {
        x: 0.0,
        y: 0.0,
        a: 0.0,
        b: 1.0,
    },
    Vec4f {
        x: 200.0,
        y: 200.0,
        a: 1.0,
        b: 1.0,
    },
];

#[derive(bincode::Encode, Debug, PartialEq)]
struct Vec4f {
    x: f32,
    y: f32,
    a: f32,
    b: f32,
}

struct State {
    device: Device,
    queue: Queue,
    size: PhysicalSize<u32>,
    surface: Surface<'static>,
    post_process_pipeline: RenderPipeline,
    framebuffer_pipeline: RenderPipeline,
    texture_format: TextureFormat,
    texture_bind_group: BindGroup,
    post_process_bind_group: BindGroup,
    uniform_position_bind_groups: Vec<BindGroup>,
    framebuffer_texture_view: TextureView,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
        //
        let instance = Instance::new(&InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .block_on()
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .block_on()
            .unwrap();
        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let surface_texture_format = surface.get_capabilities(&adapter).formats[0];

        // framebuffer texture
        let framebuffer_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("framebuffer_texture"),
            dimension: wgpu::TextureDimension::D2,
            format: surface_texture_format,
            mip_level_count: 1,
            sample_count: 1,
            size: Extent3d {
                width: 800,
                height: 600,
                ..Default::default()
            },
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[], //TextureFormat::Rgba8UnormSrgb]
        });

        let framebuffer_texture_view =
            framebuffer_texture.create_view(&TextureViewDescriptor::default());

        // sprite texture
        let sprites_img = image::load_from_memory(SPRITES).unwrap();
        let sprites_img_rgba = sprites_img.to_rgba8();
        let tex_size = Extent3d {
            width: sprites_img_rgba.dimensions().0,
            height: sprites_img_rgba.dimensions().1,
            depth_or_array_layers: 1,
        };
        let sprites_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sprite_texture"),
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            size: tex_size,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[], //TextureFormat::Rgba8UnormSrgb]
        });
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &sprites_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &sprites_img_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * tex_size.width),
                rows_per_image: Some(tex_size.height),
            },
            tex_size,
        );

        let sprites_texture_view = sprites_texture.create_view(&TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("bind_group_layout"),
            });

        // pipeline bind groups
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sprites_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        let framebuffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("framebuffer_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&framebuffer_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        // uniform position
        let uniform_position_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_position_bind_group_layout"),
            });

        let mut uniform_position_bind_groups = vec![];
        for obj_pos in OBJECTS {
            let uniform_position_buffer = device.create_buffer(&wgpu::wgt::BufferDescriptor {
                label: None,
                size: size_of::<Vec4f>() as u64,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(
                &uniform_position_buffer,
                0,
                &bincode::encode_to_vec(obj_pos, bincode::config::standard()).unwrap(),
            );

            let uniform_position_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("uniform_position_bind_group"),
                    layout: &uniform_position_bind_group_layout,
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &uniform_position_buffer,
                            size: None,
                            offset: 0,
                        }),
                    }],
                });

            uniform_position_bind_groups.push(uniform_position_bind_group);
        }

        // shader loader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader_module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shader.wgsl"))),
        });
        let post_process_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post_process_shader_module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../post_process_shader.wgsl"
            ))),
        });

        let framebuffer_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("framebuffer_pipeline_layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &uniform_position_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let framebuffer_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("framebuffer_pipeline"),
            layout: Some(&framebuffer_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                buffers: &[],
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(surface_texture_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            multiview: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            depth_stencil: None,
        });

        let post_process_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("post_process_pipeline_layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let post_process_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("post_process_pipeline"),
                layout: Some(&post_process_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &post_process_shader,
                    buffers: &[],
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &post_process_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(surface_texture_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                multiview: None,
                multisample: wgpu::MultisampleState::default(),
                cache: None,
                depth_stencil: None,
            });

        State {
            surface,
            queue,
            size,
            device,
            framebuffer_pipeline,
            post_process_pipeline,
            texture_format: surface_texture_format,
            texture_bind_group,
            post_process_bind_group: framebuffer_bind_group,
            uniform_position_bind_groups,
            framebuffer_texture_view,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        self.configure_surface();
    }

    fn configure_surface(&self) {
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.texture_format,
            width: self.size.width,
            height: self.size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![self.texture_format.add_srgb_suffix()],
        };
        self.surface.configure(&self.device, &config);
    }

    fn render(&mut self) {
        let surface_texture = self.surface.get_current_texture().unwrap();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.framebuffer_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.framebuffer_pipeline);
            render_pass.set_bind_group(0, Some(&self.texture_bind_group), &[]);

            for uniform_position_bind_group in self.uniform_position_bind_groups.clone() {
                render_pass.set_bind_group(1, Some(&uniform_position_bind_group), &[]);
                render_pass.draw(0..6, 0..1);
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post_process_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.post_process_pipeline);
            render_pass.set_bind_group(0, Some(&self.post_process_bind_group), &[]);
            render_pass.draw(0..6, 0..1);
        }
        self.queue.submit([encoder.finish()].into_iter());
        surface_texture.present();
    }
}

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(PhysicalSize::new(800, 600))
                        .with_resizable(false),
                )
                .unwrap(),
        );

        let state = State::new(window.clone());
        state.configure_surface();
        self.state = Some(state);
        self.window = Some(window.clone());
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                    window.pre_present_notify();
                }
                self.state.as_mut().unwrap().render();
            }
            WindowEvent::Resized(size) => {
                self.state.as_mut().unwrap().resize(size);
            }
            _ => (),
        }
    }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::{event_loop::EventLoopBuilder, platform::android::EventLoopBuilderExtAndroid};

    let mut event_loop: EventLoopBuilder<()> = EventLoop::with_user_event();
    let e = event_loop.with_android_app(app).build().unwrap();

    let mut app = App::default();
    e.run_app(&mut app).unwrap();
}
