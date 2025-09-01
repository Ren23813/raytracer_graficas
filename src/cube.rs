use raylib::prelude::*;
use crate::ray_intersect::{RayIntersect, HitInfo};
use crate::material::Material;
use std::f32;

pub struct Cube {
    // Center en espacio mundo, half_size en cada eje (caja AABB en espacio local)
    pub center: Vector3,
    pub half_size: Vector3,
    // Rotación en radianes (rotar primero X, luego Y) — puedes ajustar rx, ry
    pub rot_x: f32,
    pub rot_y: f32,
    pub material: Material,
}

impl Cube {
    // rota vector por X
    fn rotate_x(v: Vector3, angle: f32) -> Vector3 {
        let (s, c) = angle.sin_cos();
        Vector3::new(
            v.x,
            v.y * c - v.z * s,
            v.y * s + v.z * c,
        )
    }

    // rota vector por Y
    fn rotate_y(v: Vector3, angle: f32) -> Vector3 {
        let (s, c) = angle.sin_cos();
        Vector3::new(
            v.x * c + v.z * s,
            v.y,
            -v.x * s + v.z * c,
        )
    }

    // Aplicar rotación forward: Rx then Ry -> v' = Ry(Rx(v))
    fn rotate_forward(&self, v: Vector3) -> Vector3 {
        let v = Cube::rotate_x(v, self.rot_x);
        Cube::rotate_y(v, self.rot_y)
    }

    // Aplicar rotación inversa: inverse(Ry*Rx) = Rx(-rx) * Ry(-ry) -> aplicar Ry(-ry) then Rx(-rx)
    fn rotate_inverse(&self, v: Vector3) -> Vector3 {
        let v = Cube::rotate_y(v, -self.rot_y);
        Cube::rotate_x(v, -self.rot_x)
    }

    // safe slab como antes: maneja componentes de dirección ~ 0
    fn safe_slab(ox: f32, dx: f32, min: f32, max: f32) -> (f32, f32) {
        let eps = 1e-8;
        if dx.abs() < eps {
            if ox < min || ox > max {
                (f32::INFINITY, f32::NEG_INFINITY)
            } else {
                (f32::NEG_INFINITY, f32::INFINITY)
            }
        } else {
            let t1 = (min - ox) / dx;
            let t2 = (max - ox) / dx;
            (t1.min(t2), t1.max(t2))
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Option<HitInfo> {
        // Transformar rayo al espacio local del cubo:
        // 1) trasladar por -center
        // 2) rotar por la inversa
        let local_origin = self.rotate_inverse(*ray_origin - self.center);
        let local_direction = self.rotate_inverse(*ray_direction); // rotación sin traslación para direcciones

        // En espacio local la caja es AABB con bounds [-half, +half]
        let min = Vector3::new(-self.half_size.x, -self.half_size.y, -self.half_size.z);
        let max = Vector3::new( self.half_size.x,  self.half_size.y,  self.half_size.z);

        // Slab test en espacio local
        let (txmin, txmax) = Cube::safe_slab(local_origin.x, local_direction.x, min.x, max.x);
        let (tymin, tymax) = Cube::safe_slab(local_origin.y, local_direction.y, min.y, max.y);
        let (tzmin, tzmax) = Cube::safe_slab(local_origin.z, local_direction.z, min.z, max.z);

        // Rechazo rápido
        if txmin > tymax || tymin > txmax { return None; }
        if txmin > tzmax || tzmin > txmax { return None; }
        if tymin > tzmax || tzmin > tymax { return None; }

        let t_near = txmin.max(tymin).max(tzmin);
        let t_far  = txmax.min(tymax).min(tzmax);

        if t_near > t_far { return None; }
        if t_far < 0.0 { return None; }

        let t = if t_near >= 0.0 { t_near } else { t_far };
        if t < 0.0 { return None; }

        // Punto de impacto en espacio local
        let local_hit = local_origin + local_direction * t;

        // Determinar normal en espacio local según el slab que produjo t_near
        let eps = 1e-5;
        let mut local_normal = Vector3::new(0.0, 0.0, 0.0);

        if (t - txmin).abs() < 1e-6 {
            local_normal = if local_direction.x > 0.0 { Vector3::new(-1.0, 0.0, 0.0) } else { Vector3::new(1.0, 0.0, 0.0) };
        } else if (t - tymin).abs() < 1e-6 {
            local_normal = if local_direction.y > 0.0 { Vector3::new(0.0, -1.0, 0.0) } else { Vector3::new(0.0, 1.0, 0.0) };
        } else if (t - tzmin).abs() < 1e-6 {
            local_normal = if local_direction.z > 0.0 { Vector3::new(0.0, 0.0, -1.0) } else { Vector3::new(0.0, 0.0, 1.0) };
        } else {
            // fallback por proximidad a las caras
            if (local_hit.x - min.x).abs() < eps { local_normal = Vector3::new(-1.0, 0.0, 0.0); }
            else if (local_hit.x - max.x).abs() < eps { local_normal = Vector3::new(1.0, 0.0, 0.0); }
            else if (local_hit.y - min.y).abs() < eps { local_normal = Vector3::new(0.0, -1.0, 0.0); }
            else if (local_hit.y - max.y).abs() < eps { local_normal = Vector3::new(0.0, 1.0, 0.0); }
            else if (local_hit.z - min.z).abs() < eps { local_normal = Vector3::new(0.0, 0.0, -1.0); }
            else if (local_hit.z - max.z).abs() < eps { local_normal = Vector3::new(0.0, 0.0, 1.0); }
            else {
                local_normal = (local_hit).normalized();
            }
        }

        // Transformar punto y normal de vuelta a espacio mundo
        let world_point = self.rotate_forward(local_hit) + self.center;
        // Normales rotan con la rotación forward (rotación sin translación)
        let world_normal = self.rotate_forward(local_normal).normalized();

        Some(HitInfo {
            hit: true,
            point: world_point,
            normal: world_normal,
            distance: t, // la rotación no escala, así que t local == t mundo
            material: self.material,
        })
    }
}
