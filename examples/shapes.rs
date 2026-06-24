#[cfg(feature = "render-wgpu")]
mod app {
    use std::sync::Arc;

    use vtk_pure_rs::filters::core::sources;
    use vtk_pure_rs::render::{Actor, Camera, Renderer, Scene};
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

    impl App {
        fn new(scene: Scene) -> Self {
            Self {
                scene,
                renderer: None,
                window: None,
                mouse_pressed: false,
                last_mouse: None,
            }
        }
    }

    impl ApplicationHandler for App {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            let attrs = Window::default_attributes()
                .with_title("vtk-rs shapes")
                .with_inner_size(winit::dpi::LogicalSize::new(1024, 768));
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
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == winit::event::MouseButton::Left {
                        self.mouse_pressed = state == winit::event::ElementState::Pressed;
                        if !self.mouse_pressed {
                            self.last_mouse = None;
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if self.mouse_pressed {
                        if let Some((lx, ly)) = self.last_mouse {
                            let dx = position.x - lx;
                            let dy = position.y - ly;
                            self.scene.camera.orbit(dx * 0.5, dy * 0.5);
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        }
                        self.last_mouse = Some((position.x, position.y));
                    }
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
        let mut scene = Scene::new();

        // Sphere
        let sphere = sources::sphere::sphere(&sources::sphere::SphereParams {
            center: [-2.0, 0.0, 0.0],
            radius: 0.8,
            ..Default::default()
        });
        scene.add_actor(Actor::new(sphere).with_color(0.9, 0.2, 0.2));

        // Cube
        let cube = sources::cube::cube(&sources::cube::CubeParams {
            center: [0.0, 0.0, 0.0],
            ..Default::default()
        });
        scene.add_actor(Actor::new(cube).with_color(0.2, 0.8, 0.2));

        // Cone
        let cone = sources::cone::cone(&sources::cone::ConeParams {
            center: [2.0, 0.0, 0.0],
            ..Default::default()
        });
        scene.add_actor(Actor::new(cone).with_color(0.2, 0.4, 0.9));

        // Cylinder
        let cyl = sources::cylinder::cylinder(&sources::cylinder::CylinderParams {
            center: [4.0, 0.0, 0.0],
            ..Default::default()
        });
        scene.add_actor(Actor::new(cyl).with_color(0.9, 0.7, 0.1));

        // Arrow
        let arrow = sources::arrow::arrow(&sources::arrow::ArrowParams::default());
        scene.add_actor(Actor::new(arrow).with_color(0.8, 0.3, 0.8));

        // Camera
        scene.camera = Camera::new();
        scene.camera.reset_to_bounds([1.0, 0.0, 0.0], 8.0);

        let event_loop = EventLoop::new().unwrap();
        let mut app = App::new(scene);
        event_loop.run_app(&mut app).unwrap();
    }
}

#[cfg(feature = "render-wgpu")]
fn main() {
    app::run();
}

#[cfg(not(feature = "render-wgpu"))]
fn main() {
    println!("shapes example requires --features render-wgpu");
}
