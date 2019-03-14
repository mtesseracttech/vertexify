extern crate glium;
extern crate straal;

pub use models::*;
pub use models::ObjModel;

pub mod models;

#[cfg(test)]
mod tests {
    use super::models::*;

    #[test]
    fn load_obj_file_v() {
        let teapot = ObjModel::load_from_file("res/teapot.obj").unwrap();
        println!("{:?}", teapot);
    }

    #[test]
    fn load_obj_v_vt_vn() {
        let quad = ObjModel::load_from_file("res/quad.obj");
        println!("{:?}", quad);
    }
}
