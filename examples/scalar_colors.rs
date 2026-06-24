#[cfg(feature = "render-wgpu")]
mod app {
    use std::sync::Arc;

    use vtk_pure_rs::filters::core::{elevation, sources};
    use vtk_pure_rs::render::{Actor, Camera, ColorMap, Renderer, Scene};
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
                .with_title("vtk-rs scalar colors")
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
                            self.scene
                                .camera
                                .orbit((position.x - lx) * 0.5, (position.y - ly) * 0.5);
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

        // Sphere colored by elevation (jet colormap)
        let sphere = sources::sphere::sphere(&sources::sphere::SphereParams {
            center: [-1.5, 0.0, 0.0],
            radius: 1.0,
            theta_resolution: 32,
            phi_resolution: 32,
            ..Default::default()
        });
        let sphere_elev = elevation::elevation_z(&sphere);
        scene.add_actor(Actor::new(sphere_elev).with_scalar_coloring(ColorMap::jet(), None));

        // Cone colored by elevation (viridis)
        let cone = sources::cone::cone(&sources::cone::ConeParams {
            center: [1.5, 0.0, 0.0],
            height: 2.0,
            radius: 0.8,
            resolution: 32,
            ..Default::default()
        });
        let cone_elev = elevation::elevation_y(&cone);
        scene.add_actor(Actor::new(cone_elev).with_scalar_coloring(ColorMap::viridis(), None));

        // Plane colored by elevation (cool to warm)
        let plane = sources::plane::plane(&sources::plane::PlaneParams {
            origin: [-1.5, -2.0, -1.5],
            point1: [1.5, -2.0, -1.5],
            point2: [-1.5, -2.0, 1.5],
            x_resolution: 20,
            y_resolution: 20,
        });
        let plane_elev = elevation::elevation_x(&plane);
        scene
            .add_actor(Actor::new(plane_elev).with_scalar_coloring(ColorMap::cool_to_warm(), None));

        scene.camera = Camera::new();
        scene.camera.reset_to_bounds([0.0, 0.0, 0.0], 5.0);

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
    println!("scalar_colors example requires --features render-wgpu");
}
