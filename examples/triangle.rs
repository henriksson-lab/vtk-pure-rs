#[cfg(feature = "render-wgpu")]
mod app {
    use std::sync::Arc;

    use vtk_pure_rs::data::PolyData;
    use vtk_pure_rs::io::legacy::LegacyWriter;
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
                .with_title("vtk-rs triangle")
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
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(size) => {
                    if let Some(renderer) = &mut self.renderer {
                        renderer.resize(size.width, size.height);
                    }
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                WindowEvent::RedrawRequested => {
                    if let Some(renderer) = &mut self.renderer {
                        let _ = renderer.render(&self.scene);
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
                            if let Some(window) = &self.window {
                                window.request_redraw();
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
                    let factor = 1.0 + scroll * 0.1;
                    self.scene.camera.dolly(factor);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        }
    }

    pub fn run() {
        // Create a colored triangle
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.866, 0.0]],
            vec![[0, 1, 2]],
        );

        // Write to VTK file
        let writer = LegacyWriter::ascii();
        let dir = std::env::temp_dir().join("vtk_triangle");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("triangle.vtk");
        writer
            .write_poly_data(&path, &pd)
            .expect("failed to write VTK file");
        println!("Wrote {}", path.display());

        // Set up scene
        let mut scene = Scene::new();
        let actor = Actor::new(pd).with_color(0.2, 0.6, 1.0);
        scene.add_actor(actor);

        // Position camera
        scene.camera = Camera::new();
        scene.camera.reset_to_bounds([0.5, 0.433, 0.0], 1.2);

        // Run
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
    println!("triangle example requires --features render-wgpu");
}
