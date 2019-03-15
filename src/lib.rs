#[macro_use]
extern crate glium;
extern crate straal;

pub use models::ObjModel;

pub mod models;

#[cfg(test)]
mod tests {
    use glium::glutin;

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
}
