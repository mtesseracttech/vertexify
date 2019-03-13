extern crate straal;

pub use model_vertex::*;
pub use models::*;

pub mod models;
pub mod model_vertex;

#[cfg(test)]
mod tests {
    use super::model_vertex::*;
    use super::models::*;

    #[test]
    fn load_obj_file_v() {
        let teapot = WavefrontObjModel::load_from_file("res/teapot.obj").unwrap();
        println!("{:?}", teapot);
        assert_eq!(true, false)
    }

    #[test]
    fn load_obj_v_vt_vn() {
        let quad = WavefrontObjModel::load_from_file("res/quad.obj");
        println!("{:?}", quad);
        assert_eq!(true, false)
    }
}
