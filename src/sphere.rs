use raylib::prelude::*;

use crate::ray_intersect::RayIntersect;

pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> bool {
        let l = self.center - *ray_origin;
        let tca = l.dot(*ray_direction);
        let d2 = l.dot(l) - tca * tca;
        let radius2 = self.radius * self.radius;

        d2 <= radius2
    }
}
