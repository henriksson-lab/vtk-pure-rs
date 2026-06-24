#[cfg(feature = "render-wgpu")]
mod app {
    use std::sync::Arc;

    use vtk_pure_rs::data::{DataSet, ImageData};
    use vtk_pure_rs::filters::core::marching_cubes;
    use vtk_pure_rs::io::stl::StlWriter;
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
                .with_title("vtk-rs isosurface")
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

    /// Scalar field: a "gyroid" periodic surface
    fn gyroid(x: f64, y: f64, z: f64) -> f64 {
        (x).sin() * (y).cos() + (y).sin() * (z).cos() + (z).sin() * (x).cos()
    }

    pub fn run() {
        // Create a 3D scalar field on a regular grid
        let res = 60;
        let mut image = ImageData::with_dimensions(res, res, res);
        let extent = 4.0 * std::f64::consts::PI;
        let spacing = extent / (res - 1) as f64;
        image.set_spacing([spacing, spacing, spacing]);
        image.set_origin([-extent / 2.0, -extent / 2.0, -extent / 2.0]);

        // Evaluate scalar field at every grid point
        let n = image.num_points();
        let mut scalars = vec![0.0f64; n];
        for i in 0..n {
            let p = image.point(i);
            scalars[i] = gyroid(p[0], p[1], p[2]);
        }

        // Extract isosurface
        println!("Running marching cubes on {}^3 grid...", res);
        let iso = marching_cubes::marching_cubes(&image, &scalars, 0.0);
        println!(
            "Generated {} vertices, {} triangles",
            iso.points.len(),
            iso.polys.num_cells()
        );

        // Write to STL
        let stl_writer = StlWriter::binary();
        let dir = std::env::temp_dir().join("vtk_isosurface");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("gyroid.stl");
        stl_writer.write(&path, &iso).expect("failed to write STL");
        println!("Wrote {}", path.display());

        // Render
        let mut scene = Scene::new();
        scene.add_actor(Actor::new(iso).with_color(0.4, 0.7, 0.9));
        scene.camera = Camera::new();
        scene.camera.reset_to_bounds([0.0, 0.0, 0.0], extent);

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
    println!("isosurface example requires --features render-wgpu");
}
