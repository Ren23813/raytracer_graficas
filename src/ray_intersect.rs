use raylib::prelude::*;
use crate::material::Material;

pub struct HitInfo {
    pub hit: bool,
    pub point: Vector3,
    pub normal: Vector3,
    pub distance: f32,
    pub material: Material,
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Option<HitInfo>;
}
