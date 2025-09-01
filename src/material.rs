use raylib::prelude::*;

#[derive(Copy, Clone)]
pub struct Material {
    pub diffuse: Color,         
    pub specular: f32,          
    pub reflectivity: f32,      
    pub transparency: f32,      
    pub refractive_index: f32,  
    pub albedo: [f32; 2],     
}
