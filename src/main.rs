#[cfg(not(target_os = "android"))]
use wgpu_poc::App;
#[cfg(not(target_os = "android"))]
use winit::event_loop::EventLoop;

#[cfg(not(target_os = "android"))]
fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();

    event_loop.run_app(&mut app).unwrap();
}

#[cfg(target_os = "android")]
fn main() {}
