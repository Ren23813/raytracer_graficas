use raylib::prelude::*;

#[derive(Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub refractive_index: f32,
    pub albedo: [f32; 2],
    pub texture_path: Option<String>,
    pub emissive: Vector3,
    pub emission: f32
}
