//! Volume rendering example: GPU ray marching of a 3D scalar field.
//!
//! Creates a sphere distance field on an ImageData grid,
//! then renders it as a volume using a transfer function.

#[cfg(feature = "render-wgpu")]
mod app {
    use std::sync::Arc;

    use vtk_pure_rs::render::{Camera, ColorMap, Renderer, Scene, TransferFunction, VolumeActor};
    use vtk_pure_rs::render_wgpu::WgpuRenderer;
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::window::{Window, WindowId};

    struct App {
        scene: Scene,
        renderer: Option<WgpuRenderer>,
        window: Option<Arc<Window>>,
        mouse_pressed: bool,
        last_mouse: Option<(f64, f64)>,
    }

    impl ApplicationHandler for App {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            let attrs = Window::default_attributes()
                .with_title("vtk-rs volume rendering")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            let window = Arc::new(event_loop.create_window(attrs).unwrap());
            let renderer = pollster::block_on(WgpuRenderer::new(window.clone())).unwrap();
            self.window = Some(window);
            self.renderer = Some(renderer);
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _id: WindowId,
            event: WindowEvent,
        ) {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => {
                    if let Some(r) = &mut self.renderer {
                        r.resize(size.width, size.height);
                    }
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                WindowEvent::RedrawRequested => {
                    if let Some(r) = &mut self.renderer {
                        let _ = r.render(&self.scene);
                    }
                }
                WindowEvent::MouseInput {
                    state,
                    button: winit::event::MouseButton::Left,
                    ..
                } => {
                    self.mouse_pressed = state == winit::event::ElementState::Pressed;
                    if !self.mouse_pressed {
                        self.last_mouse = None;
                    }
                }
                WindowEvent::CursorMoved { position, .. } if self.mouse_pressed => {
                    if let Some((lx, ly)) = self.last_mouse {
                        self.scene
                            .camera
                            .orbit((position.x - lx) * 0.5, (position.y - ly) * 0.5);
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    self.last_mouse = Some((position.x, position.y));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let scroll = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => y as f64,
                        winit::event::MouseScrollDelta::PixelDelta(p) => p.y / 50.0,
                    };
                    self.scene.camera.dolly(1.0 + scroll * 0.1);
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                _ => {}
            }
        }
    }

    pub fn run() {
        // Create a 3D scalar field: sphere distance function
        let size = 32;
        let spacing = 2.0 / size as f64;
        let origin = -1.0;
        let mut scalars = Vec::with_capacity(size * size * size);

        for k in 0..size {
            for j in 0..size {
                for i in 0..size {
                    let x = origin + i as f64 * spacing;
                    let y = origin + j as f64 * spacing;
                    let z = origin + k as f64 * spacing;
                    let r = (x * x + y * y + z * z).sqrt();
                    // Gaussian blob centered at origin
                    scalars.push((-r * r * 4.0).exp());
                }
            }
        }

        // Create transfer function: cool-to-warm with gaussian opacity
        let tf = TransferFunction::gaussian(ColorMap::cool_to_warm(), 0.5, 0.2, 0.8);

        let mut volume = VolumeActor::new(
            scalars,
            [size, size, size],
            [origin, origin, origin],
            [spacing, spacing, spacing],
            tf,
        );
        volume.num_steps = 128;
        volume.opacity_scale = 2.0;

        let mut scene = Scene::new();
        scene.volumes.push(volume);
        scene.camera = Camera::new();
        scene.camera.look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0]);
        scene.background = [0.05, 0.05, 0.1, 1.0];

        println!("Volume rendering: {}^3 grid, {} steps", size, 128);
        println!("Controls: orbit (left drag), zoom (scroll)");

        let event_loop = EventLoop::new().unwrap();
        let mut app = App {
            scene,
            renderer: None,
            window: None,
            mouse_pressed: false,
            last_mouse: None,
        };
        event_loop.run_app(&mut app).unwrap();
    }
}

#[cfg(feature = "render-wgpu")]
fn main() {
    app::run();
}

#[cfg(not(feature = "render-wgpu"))]
fn main() {
    println!("volume example requires --features render-wgpu");
}
