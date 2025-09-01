// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]

use raylib::prelude::*;
use std::f32::consts::PI;

mod framebuffer;
mod ray_intersect;
mod cube;
mod material;
mod camera;

use framebuffer::Framebuffer;
use ray_intersect::{RayIntersect, HitInfo};
use cube::Cube;
use material::Material;
use camera::Camera;

// Util para comprobar si hay cualquier intersección entre origin y origin + dir*max_dist
fn intersects_any(origin: &Vector3, direction: &Vector3, objects: &[&dyn RayIntersect], max_dist: f32) -> bool {
    for obj in objects {
        if let Some(hit) = obj.ray_intersect(origin, direction) {
            if hit.distance > 1e-4 && hit.distance < max_dist {
                return true;
            }
        }
    }
    false
}

fn reflect(v: Vector3, n: Vector3) -> Vector3 {
    // reflect v around n: v - 2*(v·n)*n
    v - n * (2.0 * v.dot(n))
}

pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[&dyn RayIntersect],
) -> Color {
    let mut closest_hit: Option<HitInfo> = None;

    for object in objects {
        if let Some(hit) = object.ray_intersect(ray_origin, ray_direction) {
            if closest_hit.is_none() || hit.distance < closest_hit.as_ref().unwrap().distance {
                closest_hit = Some(hit);
            }
        }
    }

    if let Some(hit) = closest_hit {
        // Luz puntual
        let light_pos = Vector3::new(2.0, 4.0, -2.0); // juega con esta posición
        let light_color = Vector3::new(1.0, 1.0, 1.0);

        let to_light_vec = (light_pos - hit.point);
        let light_distance = to_light_vec.length();
        let dir_to_light = to_light_vec.normalized();

        // Ambient + Diffuse + Specular (Phong)
        let ambient = 0.12_f32;
        let diff = hit.normal.dot(dir_to_light).max(0.0);

        // Sombras: si hay algo entre el punto y la luz (dist < light_distance) -> en sombra
        let shadow_origin = hit.point + hit.normal * 1e-3;
        let in_shadow = intersects_any(&shadow_origin, &dir_to_light, objects, light_distance - 1e-3);
        let shadow_factor = if in_shadow { 0.2 } else { 1.0 };

        // Specular
        let view_dir = (-*ray_direction).normalized(); // dirección hacia la cámara
        let reflect_dir = reflect(-dir_to_light, hit.normal).normalized();
        let specular_strength = 0.6_f32;
        let shininess = 64.0_f32;
        let spec_angle = view_dir.dot(reflect_dir).max(0.0);
        let specular = specular_strength * spec_angle.powf(shininess);

        // Atenuación por distancia (opcional, para dar sensación 3D)
        let attenuation = 1.0 / (1.0 + 0.09 * light_distance + 0.032 * light_distance * light_distance);

        let base = Vector3::new(
            hit.material.diffuse.r as f32 / 255.0,
            hit.material.diffuse.g as f32 / 255.0,
            hit.material.diffuse.b as f32 / 255.0,
        );

        // Composición final
        let light_contrib = ambient + (1.0 - ambient) * diff * attenuation;
        let final_col = base * (light_contrib * shadow_factor) + light_color * (specular * shadow_factor * attenuation);

        Color::new(
            (final_col.x.clamp(0.0, 1.0) * 255.0) as u8,
            (final_col.y.clamp(0.0, 1.0) * 255.0) as u8,
            (final_col.z.clamp(0.0, 1.0) * 255.0) as u8,
            255,
        )
    } else {
        // Fondo
        Color::new(30, 30, 60, 255)
    }
}

// pub fn render(framebuffer: &mut Framebuffer, objects: &[&dyn RayIntersect]) {

pub fn render(framebuffer: &mut Framebuffer, objects: &[&dyn RayIntersect], camera: &Camera) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();

            let rotated_direction = camera.basis_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects);

            framebuffer.set_current_color(pixel_color);
            framebuffer.set_pixel(x, y);
        }
    }
}

fn main() {
    let window_width = 900;
    let window_height = 700;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Cubo")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as i32, window_height as i32, Color::BLACK);
    framebuffer.set_background_color(Color::new(20, 26, 40, 255));

    // SOLO el cubo (sin esferas)
     let cube = Cube {
        center: Vector3::new(1.0, 0.0, -4.0),
        half_size: Vector3::new(1.0, 1.0, 1.0),
        // rotaciones en radianes (ej: 20° en X, -30° en Y)
        rot_x: 20f32.to_radians(),
        rot_y: (-30f32).to_radians(),
        material: Material { diffuse: Color::new(160, 110, 230, 255) },
    };
    
    let cube2 = Cube {
        center: Vector3::new(2.0, 0.0, -5.0),
        half_size: Vector3::new(1.0, 1.0, 1.0),
        // rotaciones en radianes (ej: 20° en X, -30° en Y)
        rot_x: 20f32.to_radians(),
        rot_y: (-30f32).to_radians(),
        material: Material { diffuse: Color::new(160, 110, 230, 255) },
    };

    let objects_vec: Vec<&dyn RayIntersect> = vec![&cube,&cube2];
    let objects_slice: &[&dyn RayIntersect] = &objects_vec;

     let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 10.0),  // eye
        Vector3::new(0.0, 0.0, 0.0),  // center
        Vector3::new(0.0, 1.0, 0.0),  // up
    );
    let rotation_speed = PI / 100.0;

    while !window.window_should_close() {
        framebuffer.clear();

        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.orbit(0.0, rotation_speed);
        }

        render(&mut framebuffer, objects_slice, &camera);

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
