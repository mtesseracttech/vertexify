#[macro_use]
extern crate glium;
extern crate straal;

pub use models::ObjModel;

pub mod models;

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use glium::{glutin, Surface};
    use straal::{Mat3n, Mat4n, Quatn, Vec2n, Vec3n, Vec4n};

    use glutin::dpi::LogicalPosition;
    use glutin::ElementState;
    use glutin::MouseScrollDelta;
    use glutin::VirtualKeyCode;

    use super::models::*;

    #[test]
    fn load_obj_file_v() {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new();
        let context = glutin::ContextBuilder::new().with_depth_buffer(24);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        let mut teapot_model = ObjModel::load_from_file("res/teapot.obj").unwrap();
        let teapot = teapot_model.gen_glium_buffer(&display);
        println!("{:?}", teapot);
    }

    #[test]
    fn load_obj_v_vt_vn() {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new();
        let context = glutin::ContextBuilder::new().with_depth_buffer(24);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        let mut quad_model = ObjModel::load_from_file("res/quad.obj").unwrap();
        let quad = quad_model.gen_glium_buffer(&display);
        println!("{:?}", quad);
    }

    #[test]
    fn create_surface_normal() {
        let v0 = Vec3n::new(0.0, 0.0, 1.0);
        let v1 = Vec3n::new(1.0, 0.0, 1.0);
        let v2 = Vec3n::new(1.0, -1.0, 0.0);

        let n0 = generate_normal_from_face(v0, v1, v2);
        let n1 = generate_normal_from_face(v1, v2, v0);
        let n2 = generate_normal_from_face(v2, v0, v1);

        println!("{}, {}, {}", n0, n1, n2);
    }

    fn generate_normal_from_face(v0: Vec3n, v1: Vec3n, v2: Vec3n) -> Vec3n {
        let w0 = (v1 - v0).normalized();
        let w1 = (v2 - v0).normalized();
        w1.cross(w0).normalized()
    }


    #[test]
    fn normal_generation() {
        let mut events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new();
        let context = glutin::ContextBuilder::new()
            .with_depth_buffer(24)
            .with_multisampling(8);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        let draw_parameters = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,

                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        let mut teapot_model = ObjModel::load_from_file("res/teapot.obj").unwrap();
        teapot_model.generate_normals();
        let teapot = teapot_model.gen_glium_buffer(&display);

        println!("{:?}", &teapot);

        let normal_lines_program =
            glium::Program::from_source(&display,
                                        include_str!("../res/shaders/normal_lines/normal_lines.vert"),
                                        include_str!("../res/shaders/normal_lines/normal_lines.frag"),
                                        Some(include_str!("../res/shaders/normal_lines/normal_lines.geom"))).unwrap();

        let normal_colors_program =
            glium::Program::from_source(&display,
                                        include_str!("../res/shaders/normals/normals.vert"),
                                        include_str!("../res/shaders/normals/normals.frag"),
                                        None).unwrap();

        let timer = SystemTime::now();
        let mut time_current = 0.0;
        let mut time_previous = 0.0;
        let mut delta_time = 0.0;

        let view_matrix = get_view_matrix(
            &Vec3n::new(0.0, 0.0, 1.0),
            &Vec3n::new(0.0, 0.0, -1.0),
            &Vec3n::new(0.0, 1.0, 0.0),
        );

        //Mouse related debugging things
        let mut mouse_delta = Vec2n::zero();
        let mut mouse_down = false;
        let mut mouse_zoom = 2.0;
        let mut mouse_zoom_changed = false;

        let mut rotation_matrix = Mat3n::identity();

        let mut closed = false;
        while !closed {
            time_previous = time_current;
            time_current = get_time(&timer);
            delta_time = time_current - time_previous;

            let mut target = display.draw();

            let perspective_matrix = get_perspective_matrix(&Vec2n::from(target.get_dimensions()));

            if mouse_down {
                rotation_matrix.rotate_around_axis_rad(Vec3n::up(), 0.01 * mouse_delta.x);
                rotation_matrix.rotate_around_axis_rad(Vec3n::from(rotation_matrix.r0).normalized(), 0.01 * -mouse_delta.y);
            }

            let model_matrix = get_model_matrix(&Vec3n::new(0f32, 0f32, -1f32), 0.2f32) * Mat4n::from(rotation_matrix);

            let uniforms = uniform! {
                model : model_matrix,
                view: view_matrix,
                perspective : perspective_matrix,
            };

            target.clear_color_and_depth((0.01, 0.01, 0.01, 1.0), 1.0);

            //teapot.draw(&mut target, &normal_colors_program, &uniforms, &draw_parameters);
            //teapot.draw(&mut target, &normal_lines_program, &uniforms, &draw_parameters);

            target.finish().unwrap();

            mouse_delta = Vec2n::zero();
            mouse_zoom_changed = false;
            //Processing the glutin events
            events_loop.poll_events(|ev| {
                match ev {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => closed = true,
                        glutin::WindowEvent::MouseInput { state, button, .. } => {
                            if button == glutin::MouseButton::Left {
                                mouse_down = match state {
                                    ElementState::Pressed => true,
                                    ElementState::Released => false,
                                };
                            }
                        }
                        glutin::WindowEvent::MouseWheel { delta, .. } => match delta {
                            MouseScrollDelta::LineDelta(x, y) => {
                                mouse_zoom += y / 10.0;
                                mouse_zoom_changed = true;
                            }
                            MouseScrollDelta::PixelDelta(pos) => {
                                mouse_zoom += pos.y as f32;
                                mouse_zoom_changed = true;
                            }
                        },
                        _ => (), //Don't do anything for other window events
                    },
                    glutin::Event::DeviceEvent { event, .. } => match event {
                        glutin::DeviceEvent::MouseMotion { delta } => {
                            mouse_delta = Vec2n::from((delta.0 as f32, delta.1 as f32));
                        }
                        _ => (),
                    },
                    _ => (), //Don't do anything for other events
                }
            });
        }
    }

    pub fn get_view_matrix(pos: &Vec3n, dir: &Vec3n, up: &Vec3n) -> Mat4n {
        let fwd = dir.normalized();
        let rht = up.cross(fwd).normalized();
        let up = fwd.cross(rht);
        let pos = Vec3n {
            x: -pos.dot(rht),
            y: -pos.dot(up),
            z: -pos.dot(fwd),
        };

        Mat4n::new_from_vec4s(
            Vec4n::from((rht, pos.x)),
            Vec4n::from((up, pos.y)),
            Vec4n::from((fwd, pos.z)),
            Vec4n::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn get_perspective_matrix(target_dims: &Vec2n) -> Mat4n {
        let aspect_ratio = target_dims.y as f32 / target_dims.x as f32;
        let fov = std::f32::consts::PI / 3.0;
        let z_far = 1024.0;
        let z_near = 0.1;
        let f = 1.0 / (fov / 2.0).tan();

        Mat4n::new(
            f * aspect_ratio,
            0.0,
            0.0,
            0.0,
            0.0,
            f,
            0.0,
            0.0,
            0.0,
            0.0,
            (z_far + z_near) / (z_far - z_near),
            -(2.0 * z_far * z_near) / (z_far - z_near),
            0.0,
            0.0,
            1.0,
            0.0,
        )
    }

    pub fn get_model_matrix(pos: &Vec3n, scale: f32) -> Mat4n {
        Mat4n::new(
            scale, 0.0, 0.0, pos.x, 0.0, scale, 0.0, pos.y, 0.0, 0.0, scale, pos.z, 0.0, 0.0, 0.0, 1.0,
        )
    }

    pub fn get_time(timer: &SystemTime) -> f32 {
        match timer.elapsed() {
            Ok(elapsed) => {
                ((elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64) as f64
                    / 1_000_000_000.0) as f32
            }
            Err(e) => {
                println!("Error: {:?}", e);
                0.0
            }
        }
    }
}


