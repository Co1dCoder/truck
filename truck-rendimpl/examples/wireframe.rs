//! An example of using texture.

// The texture is referenced by:
// https://cc0textures.com/view?id=WoodFloor024

use truck_modeling::*;
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;
use winit::{dpi::*, event::*, event_loop::ControlFlow};
mod app;
use app::*;

struct MyApp {
    scene: Scene,
    rotate_flag: bool,
    prev_cursor: Option<Vector2>,
    light_changed: Option<std::time::Instant>,
    camera_changed: Option<std::time::Instant>,
}

impl MyApp {
    fn create_camera() -> Camera {
        let matrix = Matrix4::look_at_rh(
            Point3::new(1.5, 1.5, 1.5),
            Point3::origin(),
            Vector3::unit_y(),
        );
        Camera::perspective_camera(
            matrix.invert().unwrap(),
            Rad(std::f64::consts::PI / 4.0),
            0.1,
            40.0,
        )
    }

    fn create_cube() -> Solid {
        let v = builder::vertex(Point3::new(-0.5, -0.5, -0.5));
        let edge = builder::tsweep(&v, Vector3::unit_x());
        let mut face = builder::tsweep(&edge, Vector3::unit_y());
        let v = builder::vertex(Point3::new(0.2, 0.0, -0.5));
        let edge0 = builder::tsweep(&v, Vector3::new(-0.2, 0.2, 0.0));
        let edge1 = builder::rsweep(
            edge0.back(),
            Point3::origin(),
            Vector3::unit_z(),
            Rad(std::f64::consts::PI / 2.0),
        )
        .pop_back()
        .unwrap();
        let edge2 = builder::tsweep(edge1.back(), Vector3::new(0.2, -0.2, 0.0));
        let edge3 = builder::rsweep(
            edge2.back(),
            Point3::origin(),
            Vector3::unit_z(),
            Rad(std::f64::consts::PI / 2.0),
        )
        .pop_back()
        .unwrap();
        let edge3 = Edge::new(
            edge3.front(),
            edge0.front(),
            edge3.lock_curve().unwrap().clone(),
        );
        let wire = Wire::from(vec![
            edge3.inverse(),
            edge2.inverse(),
            edge1.inverse(),
            edge0.inverse(),
        ]);
        face.add_boundary(wire);
        builder::tsweep(&face, Vector3::unit_z())
    }
}

impl App for MyApp {
    fn init(handler: &DeviceHandler, info: AdapterInfo) -> MyApp {
        let sample_count = match info.backend {
            Backend::Vulkan => 4,
            Backend::Dx12 => 4,
            _ => 1,
        };
        let desc = SceneDescriptor {
            camera: MyApp::create_camera(),
            lights: vec![Light {
                position: Point3::new(1.0, 1.0, 1.0),
                color: Vector3::new(1.0, 1.0, 1.0),
                light_type: LightType::Point,
            }],
            sample_count,
            ..Default::default()
        };
        let mut scene = Scene::new(handler.clone(), &desc);
        let wire = scene
            .instance_creator()
            .create_wire_frame_instance(&Self::create_cube(), &Default::default());
        scene.add_object(&wire);
        MyApp {
            scene,
            rotate_flag: false,
            prev_cursor: None,
            camera_changed: None,
            light_changed: None,
        }
    }

    fn app_title<'a>() -> Option<&'a str> { Some("wired cube") }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) -> ControlFlow {
        match button {
            MouseButton::Left => {
                self.rotate_flag = state == ElementState::Pressed;
                if !self.rotate_flag {
                    self.prev_cursor = None;
                }
            }
            MouseButton::Right => {
                let (light, camera) = {
                    let desc = self.scene.descriptor_mut();
                    (&mut desc.lights[0], &desc.camera)
                };
                match light.light_type {
                    LightType::Point => {
                        light.position = camera.position();
                    }
                    LightType::Uniform => {
                        light.position = Point3::from_vec(camera.position().to_vec().normalize());
                    }
                }
            }
            _ => {}
        }
        Self::default_control_flow()
    }
    fn mouse_wheel(&mut self, delta: MouseScrollDelta, _: TouchPhase) -> ControlFlow {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                let camera = &mut self.scene.descriptor_mut().camera;
                let trans_vec = camera.eye_direction() * 0.2 * y as f64;
                camera.matrix = Matrix4::from_translation(trans_vec) * camera.matrix;
            }
            MouseScrollDelta::PixelDelta(_) => {}
        };
        Self::default_control_flow()
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) -> ControlFlow {
        if self.rotate_flag {
            let position = Vector2::new(position.x, position.y);
            if let Some(ref prev_position) = self.prev_cursor {
                let matrix = &mut self.scene.descriptor_mut().camera.matrix;
                let dir2d = &position - prev_position;
                if dir2d.so_small() {
                    return Self::default_control_flow();
                }
                let mut axis = dir2d[1] * matrix[0].truncate();
                axis += dir2d[0] * matrix[1].truncate();
                axis /= axis.magnitude();
                let angle = dir2d.magnitude() * 0.01;
                let mat = Matrix4::from_axis_angle(axis, Rad(angle));
                *matrix = mat.invert().unwrap() * *matrix;
            }
            self.prev_cursor = Some(position);
        }
        Self::default_control_flow()
    }
    fn keyboard_input(&mut self, input: KeyboardInput, _: bool) -> ControlFlow {
        let keycode = match input.virtual_keycode {
            Some(keycode) => keycode,
            None => return Self::default_control_flow(),
        };
        match keycode {
            VirtualKeyCode::P => {
                if let Some(ref instant) = self.camera_changed {
                    let time = instant.elapsed().as_secs_f64();
                    if time < 0.2 {
                        return Self::default_control_flow();
                    }
                }
                let camera = &mut self.scene.descriptor_mut().camera;
                self.camera_changed = Some(std::time::Instant::now());
                *camera = match camera.projection_type() {
                    ProjectionType::Parallel => Camera::perspective_camera(
                        camera.matrix,
                        Rad(std::f64::consts::PI / 4.0),
                        0.1,
                        40.0,
                    ),
                    ProjectionType::Perspective => {
                        Camera::parallel_camera(camera.matrix, 1.0, 0.1, 100.0)
                    }
                }
            }
            VirtualKeyCode::L => {
                if let Some(ref instant) = self.light_changed {
                    let time = instant.elapsed().as_secs_f64();
                    if time < 0.2 {
                        return Self::default_control_flow();
                    }
                }
                self.light_changed = Some(std::time::Instant::now());
                let (light, camera) = {
                    let desc = self.scene.descriptor_mut();
                    (&mut desc.lights[0], &desc.camera)
                };
                *light = match light.light_type {
                    LightType::Point => {
                        let position = Point3::from_vec(camera.position().to_vec().normalize());
                        Light {
                            position,
                            color: Vector3::new(1.0, 1.0, 1.0),
                            light_type: LightType::Uniform,
                        }
                    }
                    LightType::Uniform => {
                        let position = camera.position();
                        Light {
                            position,
                            color: Vector3::new(1.0, 1.0, 1.0),
                            light_type: LightType::Point,
                        }
                    }
                }
            }
            _ => {}
        }
        Self::default_control_flow()
    }

    fn render(&mut self, frame: &SwapChainFrame) { self.scene.render_scene(&frame.output.view); }
}

fn main() { MyApp::run(); }
