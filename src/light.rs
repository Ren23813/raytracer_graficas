use raylib::prelude::*;

#[derive(Clone, Copy)]
pub struct Light {
    pub position: Vector3,
    pub color: Vector3,
    pub intensity: f32,
}
