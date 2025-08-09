use raylib::prelude::*;
use crate::ray_intersect::{RayIntersect, HitInfo};
use crate::material::Material;
use std::f32;

pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
    pub material: Material
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Option<HitInfo> {
        let l = self.center - *ray_origin;
        let tca = l.dot(*ray_direction);
        let d2 = l.dot(l) - tca * tca;
        let radius2 = self.radius * self.radius;

        if d2 > radius2 {
            return None;
        }

        let thc = (radius2 - d2).sqrt();
        let t0 = tca - thc;
        let t1 = tca + thc;

        let t = if t0 < 0.0 { t1 } else { t0 };
        if t < 0.0 {
            return None;
        }

        let hit_point = *ray_origin + *ray_direction * t;
        let normal = (hit_point - self.center).normalized();

        // Devolvemos la referencia al objeto (`self`)
        Some(HitInfo {
            hit: true,
            point: hit_point,
            normal,
            distance: t,
            object: self,  // Pasamos la referencia al objeto
        })
    }
}


