use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::vec::*;

use glium::Surface;
use straal::FloatType;
use straal::vec3::Vec3;

use super::*;

#[derive(Debug)]
pub struct ObjModel {
    indices: Vec<u32>,
    vertices: Vec<Vertex>,
    normals: Vec<Normal>,
    tex_coords: Vec<UV>,
}

#[derive(Debug)]
pub struct GliumBuffers {
    pub indices: glium::IndexBuffer<u32>,
    pub vertices: glium::VertexBuffer<Vertex>,
    pub normals: glium::VertexBuffer<Normal>,
    pub tex_coords: glium::VertexBuffer<UV>,
    pub has_normals: bool,
    pub has_tex_coords: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: straal::Vec3n,
}
implement_vertex!(Vertex, position);

#[derive(Copy, Clone, Debug)]
pub struct Normal {
    normal: straal::Vec3n,
}
implement_vertex!(Normal, normal);

#[derive(Copy, Clone, Debug)]
pub struct UV {
    tex_coords: straal::Vec2n,
}
implement_vertex!(UV, tex_coords);


#[derive(Hash, PartialEq, Eq)]
struct FaceIndexTriplet {
    pub v: usize,
    pub n: Option<usize>,
    pub uv: Option<usize>,
}

#[derive(Debug)]
enum TripletType {
    VertexOnly,
    VertexTexture,
    VertexNormal,
    VertexTextureNormal,
}


#[derive(Debug)]
pub struct ModelLoadingError {
    file_path: String,
    message: String,
    buffer_reader_error: Option<io::Error>,
}

impl fmt::Display for models::ModelLoadingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not process the obj file")
    }
}

impl error::Error for models::ModelLoadingError {
    fn description(&self) -> &str { "Obj Error" }
}

impl ObjModel {
    pub fn load_from_file(file_path: &str) -> Result<ObjModel, ModelLoadingError> {
        let file: File = match File::open(file_path) {
            Ok(file) => file,
            Err(e) => {
                println!("{}", e);
                return Err(ModelLoadingError {
                    file_path: file_path.to_string(),
                    message: "Could not open file!".to_string(),
                    buffer_reader_error: None,
                });
            }
        };

        let mut vertices: Vec<straal::Vec3n> = Vec::new();
        let mut normals: Vec<straal::Vec3n> = Vec::new();
        let mut uvs: Vec<straal::Vec2n> = Vec::new();
        let mut faces: Vec<FaceIndexTriplet> = Vec::new();

        let mut line_no = 1;
        for line in BufReader::new(file).lines() {
            match line {
                Ok(line) => {
                    if !line.is_empty() {
                        let tokens: Vec<&str> = line.split_whitespace().collect();
                        if !tokens.is_empty() {
                            match tokens[0] {
                                "v" => {
                                    //Parse vertex
                                    //v x y z
                                    let mut parsed = tokens.iter().skip(1).flat_map(|s: &&str| s.parse());

                                    vertices.push(straal::Vec3 {
                                        x: parsed.next().unwrap(),
                                        y: parsed.next().unwrap(),
                                        z: parsed.next().unwrap(),
                                    });
                                }
                                "vn" => {
                                    //Parse vertex normal
                                    //vn x y z
                                    let mut parsed = tokens.iter().skip(1).flat_map(|s: &&str| s.parse());
                                    normals.push(straal::Vec3 {
                                        x: parsed.next().unwrap(),
                                        y: parsed.next().unwrap(),
                                        z: parsed.next().unwrap(),
                                    }.normalized());
                                }
                                "vt" => {
                                    //Parse vertex texture coordinate
                                    //vt x y
                                    let mut parsed = tokens.iter().skip(1).flat_map(|s: &&str| s.parse());
                                    uvs.push(straal::Vec2 {
                                        x: parsed.next().unwrap(),
                                        y: parsed.next().unwrap(),
                                    });
                                }
                                "f" => {
                                    //Parse polygon face
                                    //f v1 v2 v3
                                    //f v1/vt1 v2/vt2 v3/vt3
                                    //f v1//vn1 v2//vn2 v3//vn3
                                    //f v1/vt1/vn1 v2/vt2/vn2 v3/vt3/vn3
                                    faces.append(&mut ObjModel::parse_face_line(&tokens));
                                }
                                "#" => { /*Comment, not much to do here*/ }
                                "mtllib" => { /*Material file location*/ }
                                "usemtl" => { /*Use material for the element following this statement*/ }
                                "o" => { /*Object name*/ }
                                "g" => { /*Group name*/ }
                                "s" => { /*Smoothing enable/disable for smoothing group*/ }
                                token => {
                                    return Err(ModelLoadingError {
                                        file_path: file_path.to_string(),
                                        message: format!("Could not identify token {} for line: {}", token, line),
                                        buffer_reader_error: None,
                                    });
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(ModelLoadingError {
                        file_path: file_path.to_string(),
                        message: format!("Could not read line {}", line_no),
                        buffer_reader_error: Some(e),
                    });
                }
            }
            line_no += 1;
        }

        let mut model = ObjModel {
            indices: Vec::new(),
            vertices: Vec::new(),
            normals: Vec::new(),
            tex_coords: Vec::new(),
        };

        let mut mapped_triplets: HashMap<FaceIndexTriplet, u32> = HashMap::new();


        for face_index_triplet in faces {
            match mapped_triplets.get(&face_index_triplet) {
                None => {
                    let index = mapped_triplets.len() as u32;

                    model.indices.push(index);
                    model.vertices.push(Vertex { position: vertices[face_index_triplet.v - 1] });

                    if face_index_triplet.uv.is_some() {
                        model.tex_coords.push(UV { tex_coords: uvs[face_index_triplet.uv.unwrap() - 1] });
                    }
                    if face_index_triplet.n.is_some() {
                        model.normals.push(Normal { normal: normals[face_index_triplet.n.unwrap() - 1] });
                    }

                    mapped_triplets.insert(face_index_triplet, index);
                }
                Some(i) => {
                    model.indices.push(*i);
                }
            }
        }

        return Ok(model);
    }

    fn parse_face_line(tokens: &Vec<&str>) -> Vec<FaceIndexTriplet> {
        let triplet_vec: Vec<&str> = tokens.iter().skip(1).flat_map(|s: &&str| s.split("/")).collect();
        let mut parsed = triplet_vec.iter().flat_map(|s: &&str| s.parse());

        let mut triangle = Vec::new();
        match ObjModel::get_face_triplet_type(&triplet_vec) {
            TripletType::VertexOnly => {
                for _i in 0..3 {
                    triangle.push(FaceIndexTriplet {
                        v: parsed.next().unwrap(),
                        uv: None,
                        n: None,
                    });
                }
            }
            TripletType::VertexTexture => {
                for _i in 0..3 {
                    triangle.push(FaceIndexTriplet {
                        v: parsed.next().unwrap(),
                        uv: Some(parsed.next().unwrap()),
                        n: None,
                    });
                }
            }
            TripletType::VertexNormal => {
                //Need to filter the empty entries out
                let mut parsed = triplet_vec.iter().skip_while(|s| s.is_empty()).flat_map(|s| s.parse());
                for _i in 0..3 {
                    triangle.push(FaceIndexTriplet {
                        v: parsed.next().unwrap(),
                        uv: None,
                        n: Some(parsed.next().unwrap()),
                    });
                }
            }
            TripletType::VertexTextureNormal => {
                for _i in 0..3 {
                    triangle.push(FaceIndexTriplet {
                        v: parsed.next().unwrap(),
                        uv: Some(parsed.next().unwrap()),
                        n: Some(parsed.next().unwrap()),
                    });
                }
            }
        }
        triangle
    }

    fn get_face_triplet_type(triplets: &Vec<&str>) -> TripletType {
        match triplets.len() {
            3 => TripletType::VertexOnly,
            6 => TripletType::VertexTexture,
            9 => {
                match triplets[1].len() {
                    0 => TripletType::VertexNormal,
                    _ => TripletType::VertexTextureNormal
                }
            }
            _ => panic!("Unknown face triplet type encountered")
        }
    }

    pub fn generate_normals(&mut self) {
        let mut normals = Vec::with_capacity(self.indices.len());
        let mut i = 0;
        while i < self.indices.len() {
            let v0 = self.vertices[self.indices[i] as usize].position;
            let v1 = self.vertices[self.indices[i + 1] as usize].position;
            let v2 = self.vertices[self.indices[i + 2] as usize].position;
            let n = ObjModel::three_vertices_to_normal(v0, v1, v2);
            normals.push(n);
            normals.push(n);
            normals.push(n);
            i += 3;
        }

        let mut normals_map = HashMap::with_capacity(self.vertices.len());

        for pair in self.indices.iter().zip(normals.iter()) {
            if !normals_map.contains_key(pair.0) {
                normals_map.insert(*pair.0, Vec::new());
            }
            normals_map.get_mut(pair.0).unwrap().push(*pair.1);
        }

        let mut final_normals = vec![Normal { normal: Vec3::zero() }; self.vertices.len()];
        //let mut final_normals = Vec::with_capacity(self.vertices.len());
        for normal_count_pair in normals_map {
            //println!("Normals to be combined: {}", normal_count_pair.1.len());
            //println!("Input normals: ");
            for input_normal in &normal_count_pair.1 {
                println!("{}", input_normal)
            }
            let mut proper_normal = Vec3::zero();
            for partial_normal in normal_count_pair.1 {
                proper_normal += partial_normal;
            }
            proper_normal = proper_normal.normalized();
            //println!("Proper normal: {}", proper_normal);
            final_normals[normal_count_pair.0 as usize] = Normal { normal: proper_normal };
        }
        //println!("Vertex size: {}", self.vertices.len());
        //println!("Normal size: {}", final_normals.len());

        self.normals = final_normals;
    }


    //v0 is the point from where the normal is calculated
    fn three_vertices_to_normal(v0: straal::Vec3n, v1: straal::Vec3n, v2: straal::Vec3n) -> straal::Vec3n {
        let w0 = (v1 - v0).normalized();
        let w1 = (v2 - v0).normalized();
        w0.cross(w1).normalized()
    }

    pub fn gen_glium_buffer(&self, display: &glium::Display) -> GliumBuffers {
        GliumBuffers {
            indices: glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &self.indices).unwrap(),
            vertices: glium::VertexBuffer::new(display, &self.vertices).unwrap(),
            normals: glium::VertexBuffer::new(display, &self.normals).unwrap(),
            tex_coords: glium::VertexBuffer::new(display, &self.tex_coords).unwrap(),
            has_normals: !self.normals.is_empty(),
            has_tex_coords: !self.tex_coords.is_empty(),
        }
    }
}

impl GliumBuffers {
    pub fn draw<U>(&self, target: &mut glium::Frame, program: &glium::Program, uniforms: &U, draw_params: &glium::DrawParameters) where U: glium::uniforms::Uniforms {
        if self.has_tex_coords && self.has_normals {
            target.draw((&self.vertices, &self.normals, &self.tex_coords), &self.indices, program, uniforms, draw_params).unwrap();
        } else if self.has_tex_coords && !self.has_normals {
            target.draw((&self.vertices, &self.tex_coords), &self.indices, program, uniforms, draw_params).unwrap();
        } else if !self.has_tex_coords && self.has_normals {
            target.draw((&self.vertices, &self.normals), &self.indices, program, uniforms, draw_params).unwrap();
        } else {
            target.draw(&self.vertices, &self.indices, program, uniforms, draw_params).unwrap();
        }
    }
}



