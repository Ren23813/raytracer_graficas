use raylib::prelude::*;

use crate::sphere::Sphere;

pub struct HitInfo<'a> {
    pub hit: bool,
    pub point: Vector3,
    pub normal: Vector3,
    pub distance: f32,
    pub object: &'a Sphere,  // Aquí agregamos la referencia al objeto
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Option<HitInfo>;
}
