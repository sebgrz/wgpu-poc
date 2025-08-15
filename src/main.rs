use std::{rc::Rc, sync::Arc};

use pollster::FutureExt as _;
use wgpu::{
    Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceTarget, TextureFormat, TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct State {
    device: Device,
    queue: Queue,
    size: PhysicalSize<u32>,
    surface: Surface<'static>,
    texture_format: TextureFormat,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
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
        let texture_format = surface.get_capabilities(&adapter).formats[0];

        State {
            surface,
            queue,
            size,
            device,
            texture_format,
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
        surface_texture.present();
    }
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = State::new(window.clone());
        state.configure_surface();
        self.state = Some(state);
        self.window = Some(window.clone());
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
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
fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();

    event_loop.run_app(&mut app).unwrap();
}
