#[cfg(feature = "render-wgpu")]
mod app {
    //! Showcase example demonstrating advanced rendering features:
    //! - PBR materials (metallic sphere, rough cube)
    //! - Transparency (translucent cone)
    //! - Edge overlay (wireframe on cylinder)
    //! - Scalar bar with color legend
    //! - Axes orientation widget
    //! - Scalar coloring with elevation
    //! - Mouse interaction: orbit (left), pan (middle), zoom (scroll)

    use std::sync::Arc;

    use vtk_pure_rs::filters::core::{elevation, sources};
    use vtk_pure_rs::render::{
        Actor, AxesWidget, Camera, ColorMap, Material, Renderer, Representation, ScalarBar, Scene,
    };
    use vtk_pure_rs::render_wgpu::WgpuRenderer;
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::window::{Window, WindowId};

    struct App {
        scene: Scene,
        renderer: Option<WgpuRenderer>,
        window: Option<Arc<Window>>,
        left_pressed: bool,
        middle_pressed: bool,
        last_mouse: Option<(f64, f64)>,
    }

    impl App {
        fn new(scene: Scene) -> Self {
            Self {
                scene,
                renderer: None,
                window: None,
                left_pressed: false,
                middle_pressed: false,
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
                .with_title("vtk-rs showcase")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 960));
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
                    let pressed = state == winit::event::ElementState::Pressed;
                    match button {
                        winit::event::MouseButton::Left => {
                            self.left_pressed = pressed;
                            if !pressed {
                                self.last_mouse = None;
                            }
                        }
                        winit::event::MouseButton::Middle => {
                            self.middle_pressed = pressed;
                            if !pressed {
                                self.last_mouse = None;
                            }
                        }
                        _ => {}
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if self.left_pressed || self.middle_pressed {
                        if let Some((lx, ly)) = self.last_mouse {
                            let dx = position.x - lx;
                            let dy = position.y - ly;
                            if self.left_pressed {
                                self.scene.camera.orbit(dx * 0.5, dy * 0.5);
                            } else {
                                self.scene.camera.pan(dx, dy);
                            }
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        }
                        self.last_mouse = Some((position.x, position.y));
                    }
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == winit::event::ElementState::Pressed {
                        use winit::keyboard::{Key, NamedKey};
                        match event.logical_key {
                            Key::Character(ref c) => match c.as_str() {
                                "r" | "R" => {
                                    self.scene.camera.reset_to_bounds([1.0, 0.0, 0.0], 12.0);
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                }
                                "w" | "W" => {
                                    for actor in &mut self.scene.actors {
                                        actor.representation = Representation::Wireframe;
                                    }
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                }
                                "s" | "S" => {
                                    for actor in &mut self.scene.actors {
                                        actor.representation = Representation::Surface;
                                    }
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                }
                                "p" | "P" => {
                                    for actor in &mut self.scene.actors {
                                        actor.representation = Representation::Points;
                                    }
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                }
                                "f" | "F" => {
                                    for actor in &mut self.scene.actors {
                                        actor.material.flat_shading = !actor.material.flat_shading;
                                    }
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                }
                                "e" | "E" => {
                                    for actor in &mut self.scene.actors {
                                        actor.material.edge_visibility =
                                            !actor.material.edge_visibility;
                                    }
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                }
                                "b" | "B" => {
                                    for actor in &mut self.scene.actors {
                                        actor.material.backface_culling =
                                            !actor.material.backface_culling;
                                    }
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                }
                                _ => {}
                            },
                            Key::Named(NamedKey::Escape) => event_loop.exit(),
                            _ => {}
                        }
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

        // PBR metallic sphere
        let sphere = sources::sphere::sphere(&sources::sphere::SphereParams {
            center: [-3.0, 0.0, 0.0],
            radius: 0.8,
            theta_resolution: 32,
            phi_resolution: 32,
            ..Default::default()
        });
        let mut sphere_actor = Actor::new(sphere).with_color(0.95, 0.7, 0.3);
        sphere_actor.material = Material::pbr_metal(0.2);
        scene.add_actor(sphere_actor);

        // PBR rough dielectric cube
        let cube = sources::cube::cube(&sources::cube::CubeParams {
            center: [-1.0, 0.0, 0.0],
            ..Default::default()
        });
        let mut cube_actor = Actor::new(cube).with_color(0.2, 0.6, 0.9);
        cube_actor.material = Material::pbr_dielectric(0.8);
        scene.add_actor(cube_actor);

        // Translucent cone
        let cone = sources::cone::cone(&sources::cone::ConeParams {
            center: [1.0, 0.0, 0.0],
            ..Default::default()
        });
        let mut cone_actor = Actor::new(cone).with_color(0.9, 0.2, 0.2);
        cone_actor.opacity = 0.5;
        scene.add_actor(cone_actor);

        // Wireframe + edges cylinder
        let cyl = sources::cylinder::cylinder(&sources::cylinder::CylinderParams {
            center: [3.0, 0.0, 0.0],
            ..Default::default()
        });
        let mut cyl_actor = Actor::new(cyl).with_color(0.3, 0.8, 0.3);
        cyl_actor.material = Material::default().with_edges();
        scene.add_actor(cyl_actor);

        // Elevation-colored sphere with scalar bar
        let elev_sphere = sources::sphere::sphere(&sources::sphere::SphereParams {
            center: [5.0, 0.0, 0.0],
            radius: 0.8,
            theta_resolution: 32,
            phi_resolution: 32,
            ..Default::default()
        });
        let elev_data = elevation::elevation_z(&elev_sphere);
        let elev_actor = Actor::new(elev_data).with_scalar_coloring(ColorMap::viridis(), None);
        scene.add_actor(elev_actor);

        // Scalar bar
        scene.add_scalar_bar(ScalarBar::new(
            "Elevation",
            ColorMap::viridis(),
            [-0.8, 0.8],
        ));

        // Axes widget
        scene.axes_widget = Some(AxesWidget::default());

        // Camera
        scene.camera = Camera::new();
        scene.camera.reset_to_bounds([1.0, 0.0, 0.0], 12.0);

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
    println!("showcase example requires --features render-wgpu");
}
