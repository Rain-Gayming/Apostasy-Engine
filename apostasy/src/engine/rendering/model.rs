use std::{fs, io};

use crate as apostasy;
use apostasy_macros::Component;

const MODEL_LOCATION: &str = "res/models/";

#[derive(Component)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Loads a model, path should be the file name, default path is "/res/models/"
pub fn load_model(path: &str) -> Model {
    let path = format!("{}{}", MODEL_LOCATION, path);
    let file = fs::File::open(path).unwrap();
    let reader = io::BufReader::new(file);
    let _gltf = gltf::Gltf::from_reader(reader).unwrap();
    Model { meshes: Vec::new() }
}

pub struct Vertex {}
