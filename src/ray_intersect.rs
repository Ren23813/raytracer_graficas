use raylib::prelude::*;
use crate::material::Material;

pub struct HitInfo {
    pub hit: bool,
    pub point: Vector3,        // punto en espacio mundo
    pub local_point: Vector3,  // punto en espacio local del objeto (útil para UVs)
    pub local_half_size: Vector3, // <-- nuevo: half_size del objeto en local (útil para mapeo)
    pub normal: Vector3,       // normal en espacio mundo
    pub local_normal: Vector3, // normal en espacio local (útil para decidir cara)
    pub distance: f32,
    pub material: Material,
    pub texture_repeat: Vector2, // cuantas repeticiones aplicar (x: u, y: v)
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Option<HitInfo>;
}
