extern crate regex;

use std::collections::HashMap;
use std::error;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::vec::*;

use regex::Regex;

use super::*;

#[derive(Debug)]
pub struct WavefrontObjModel {
    indices: Vec<usize>,
    vertices: Vec<ModelVertex>,
}

#[derive(Hash, PartialEq, Eq)]
struct FaceIndexTriplet {
    pub v: usize,
    pub uv: usize,
    pub n: usize,
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
    fn description(&self) -> &str {
        "Obj Error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl WavefrontObjModel {
    pub fn load_from_file(file_path: &str) -> Result<WavefrontObjModel, ModelLoadingError> {
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

        let mut model = WavefrontObjModel {
            indices: Vec::new(),
            vertices: Vec::new(),
        };

        let mut mapped_triplets: HashMap<FaceIndexTriplet, usize> = HashMap::new();

        let mut vertices: Vec<straal::Vec3> = Vec::new();
        let mut normals: Vec<straal::Vec3> = Vec::new();
        let mut uvs: Vec<straal::Vec2> = Vec::new();

        let mut line_no = 1;
        for line in BufReader::new(file).lines() {
            match line {
                Ok(line) => {
                    if line.len() >= 10 {
                        let tokens: Vec<&str> = line.split_whitespace().collect();
                        if !tokens.is_empty() {
                            match tokens[0] {
                                "v" => {
                                    //Vertex position
                                    let mut parsed = tokens.iter().skip(1).flat_map(|s: &&str| s.parse());
                                    vertices.push(straal::Vec3 {
                                        x: parsed.next().unwrap(),
                                        y: parsed.next().unwrap(),
                                        z: parsed.next().unwrap(),
                                    })
                                }
                                "vn" => {
                                    //Vertex normal
                                    let mut parsed = tokens.iter().skip(1).flat_map(|s: &&str| s.parse());
                                    normals.push(straal::Vec3 {
                                        x: parsed.next().unwrap(),
                                        y: parsed.next().unwrap(),
                                        z: parsed.next().unwrap(),
                                    })
                                }
                                "vt" => {
                                    //Vertex texture coordinate
                                    let mut parsed = tokens.iter().skip(1).flat_map(|s: &&str| s.parse());
                                    uvs.push(straal::Vec2 {
                                        x: parsed.next().unwrap(),
                                        y: parsed.next().unwrap(),
                                    })
                                }
                                "f" => {
                                    let mut triplets: Vec<&str> = tokens.iter().skip(1).flat_map(|s: &&str| s.split("/")).collect();
                                    match triplets.len() {
                                        3 => {
                                            //Only indices for the positions, no normals or texture coordinates
                                            let mut parsed = triplets.iter().flat_map(|s: &&str| s.parse());
                                            for i in 0..3 {
                                                let face_index_triplet = FaceIndexTriplet {
                                                    v: parsed.next().unwrap(),
                                                    uv: 0,
                                                    n: 0,
                                                };

                                                match mapped_triplets.get(&face_index_triplet) {
                                                    None => {
                                                        let index = mapped_triplets.len();

                                                        model.indices.push(index);
                                                        model.vertices.push(ModelVertex {
                                                            position: vertices[face_index_triplet.v - 1],
                                                            normal: normals[face_index_triplet.n - 1],
                                                            uvs: uvs[face_index_triplet.uv - 1],
                                                        });

                                                        mapped_triplets.insert(face_index_triplet, index);
                                                    }
                                                    Some(i) => {
                                                        model.indices.push(*i);
                                                    }
                                                }
                                            }
                                        }
                                        9 => {
                                            //Support for all 3 vertex types
                                            let mut parsed = triplets.iter().flat_map(|s: &&str| s.parse());
                                            for i in 0..3 {
                                                let face_index_triplet = FaceIndexTriplet {
                                                    v: parsed.next().unwrap(),
                                                    uv: parsed.next().unwrap(),
                                                    n: parsed.next().unwrap(),
                                                };

                                                match mapped_triplets.get(&face_index_triplet) {
                                                    None => {
                                                        let index = mapped_triplets.len();

                                                        model.indices.push(index);
                                                        model.vertices.push(ModelVertex {
                                                            position: vertices[face_index_triplet.v - 1],
                                                            normal: normals[face_index_triplet.n - 1],
                                                            uvs: uvs[face_index_triplet.uv - 1],
                                                        });

                                                        mapped_triplets.insert(face_index_triplet, index);
                                                    }
                                                    Some(i) => {
                                                        model.indices.push(*i);
                                                    }
                                                }
                                            }
                                        }
                                        (n) => {
                                            return Err(ModelLoadingError {
                                                file_path: file_path.to_string(),
                                                message: format!("Unknown face index count {} on line: {}", n, line),
                                                buffer_reader_error: None,
                                            });
                                        }
                                    }
                                }
                                "mtllib" => { /* not sure what to do with this yet*/ }
                                "usemtl" => { /* not sure what to do with this yet*/ }
                                "#" => { /* comment, not an error, so it's ignored */ }
                                (token) => {
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
        Ok(model)
    }
}



