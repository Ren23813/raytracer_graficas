use raylib::prelude::*;
use crate::material::Material;

pub struct HitInfo {
    pub hit: bool,
    pub point: Vector3,        // punto en espacio mundo
    pub local_point: Vector3,  // punto en espacio local del objeto (útil para UVs)
    pub normal: Vector3,       // normal en espacio mundo
    pub local_normal: Vector3, // normal en espacio local (útil para decidir cara)
    pub distance: f32,
    pub material: Material,
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Option<HitInfo>;
}
