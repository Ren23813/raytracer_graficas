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
mod light;
mod textures;

use framebuffer::Framebuffer;
use ray_intersect::{RayIntersect, HitInfo};
use cube::Cube;
use material::Material;
use camera::Camera;
use light::Light;
use textures::TextureManager;

// Util para comprobar si hay cualquier intersección entre origin y origin + dir*max_dist
fn intersects_any(
    origin: &Vector3,
    direction: &Vector3,
    objects: &[&dyn RayIntersect],
    max_dist: f32,
) -> bool {
    for obj in objects {
        if let Some(hit) = obj.ray_intersect(origin, direction) {
            if hit.distance < max_dist {
                return true;
            }
        }
    }
    false
}

fn reflect(i: &Vector3, n: &Vector3) -> Vector3 {
    *i - *n * 2.0 * i.dot(*n)
}

pub fn refract(incident: &Vector3, normal: &Vector3, refractive_index: f32) -> Vector3 {
    // Implementation of Snell's Law for refraction.
    // It calculates the direction of a ray as it passes from one medium to another.

    // `cosi` is the cosine of the angle between the incident ray and the normal.
    // We clamp it to the [-1, 1] range to avoid floating point errors.
    let mut cosi = incident.dot(*normal).max(-1.0).min(1.0);

    // `etai` is the refractive index of the medium the ray is currently in.
    // `etat` is the refractive index of the medium the ray is entering.
    // `n` is the normal vector, which may be flipped depending on the ray's direction.
    let mut etai = 1.0; // Assume we are in Air (or vacuum) initially
    let mut etat = refractive_index;
    let mut n = *normal;

    if cosi > 0.0 {
        // The ray is inside the medium (e.g., glass) and going out into the air.
        // We need to swap the refractive indices.
        std::mem::swap(&mut etai, &mut etat);
        // We also flip the normal so it points away from the medium.
        n = -n;
    } else {
        // The ray is outside the medium and going in.
        // We need a positive cosine for the calculation, so we negate it.
        cosi = -cosi;
    }

    // `eta` is the ratio of the refractive indices (n1 / n2).
    let eta = etai / etat;
    // `k` is a term derived from Snell's law that helps determine if total internal reflection occurs.
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);

    if k < 0.0 {
        // If k is negative, it means total internal reflection has occurred.
        // There is no refracted ray, so we return None.
        Vector3::zero()
    } else {
        // If k is non-negative, we can calculate the direction of the refracted ray.
        *incident * eta + n * (eta * cosi - k.sqrt())
    }
}



fn get_cube_uv(hit_point: Vector3, normal: Vector3) -> (f32, f32) {
    let (u, v) = if normal.x.abs() > 0.5 {
        // Cara derecha o izquierda
        ((hit_point.z + 0.5), (hit_point.y + 0.5))
    } else if normal.y.abs() > 0.5 {
        // Cara superior o inferior
        ((hit_point.x + 0.5), (hit_point.z + 0.5))
    } else {
        // Cara delantera o trasera
        ((hit_point.x + 0.5), (hit_point.y + 0.5))
    };

    (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

fn map_uv_for_cube(hit_point: &Vector3, normal: &Vector3) -> Option<(f32, f32)> {
    // Asume cubo centrado en origen y de tamaño 2 (half_size = 1)
    let p = *hit_point;
    let (u, v) = if normal.x.abs() > 0.9 {
        ((p.z + 1.0) * 0.5, (p.y + 1.0) * 0.5)
    } else if normal.y.abs() > 0.9 {
        ((p.x + 1.0) * 0.5, (p.z + 1.0) * 0.5)
    } else if normal.z.abs() > 0.9 {
        ((p.x + 1.0) * 0.5, (p.y + 1.0) * 0.5)
    } else {
        return None;
    };

    Some((u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)))
}



// firma actualizada: ahora recibe texture_manager: &TextureManager
pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[&dyn RayIntersect],
    light: &Light,
    depth: u32,
    texture_manager: &TextureManager,
) -> Vector3 {
    if depth > 3 {
        return Vector3::new(0.1, 0.1, 0.2); // sky
    }

    let mut closest_hit: Option<HitInfo> = None;
    for object in objects {
        if let Some(hit) = object.ray_intersect(ray_origin, ray_direction) {
            if closest_hit.is_none() || hit.distance < closest_hit.as_ref().unwrap().distance {
                closest_hit = Some(hit);
            }
        }
    }

    if let Some(hit) = closest_hit {
        let light_dir = (light.position - hit.point).normalized();
        let view_dir = (*ray_origin - hit.point).normalized();
        let m = hit.material;

        // sombra
        let shadow_origin: Vector3 = hit.point + hit.normal * 1e-3;
        let in_shadow = intersects_any(&shadow_origin, &light_dir, objects, (light.position - hit.point).length() - 1e-3);
        let shadow_intensity = if in_shadow { 0.8 } else { 0.0 };
        let light_intensity = light.intensity * (1.0 - shadow_intensity);

        // color base desde material
        let mut base_color = Vector3::new(
            m.diffuse.r as f32 / 255.0,
            m.diffuse.g as f32 / 255.0,
            m.diffuse.b as f32 / 255.0,
        );

        // Si hay textura, usamos UV local (si existe) y sampleamos con texture_manager.sample_uv
        if let Some(texture_path) = &m.texture_path {
            // asumimos cubo -> mapear con las coordenadas locales del hit
            if let Some((u, v)) = map_uv_for_cube(&hit.local_point, &hit.local_normal) {
                base_color = texture_manager.sample_uv(texture_path, u, v);
            }
        }

        let diffuse_intensity = hit.normal.dot(light_dir).max(0.0) * light_intensity;
        let diffuse = base_color * diffuse_intensity;

        // specular con la luz
        let reflect_dir = reflect(&-light_dir, &-hit.normal).normalized();
        let specular_intensity = view_dir.dot(reflect_dir).max(0.0).powf(m.specular) * light_intensity;
        let specular = light.color * specular_intensity;

        // Reflection
        let mut reflection_color = Vector3::new(0.1, 0.1, 0.2);
        if m.reflectivity > 0.0 {
            let rdir = reflect(ray_direction, &hit.normal).normalized();
            let rorigin = hit.point + hit.normal * 1e-3;
            reflection_color = cast_ray(&rorigin, &rdir, objects, light, depth + 1, texture_manager);
        }

        // Refraction
        let mut refraction_color = Vector3::zero();
        if m.transparency > 0.0 {
            let refr = refract(ray_direction, &hit.normal, m.refractive_index).normalized();
            let rorigin = hit.point - hit.normal * 1e-3;
            refraction_color = cast_ray(&rorigin, &refr, objects, light, depth + 1, texture_manager);
        }

        let color = diffuse * m.albedo[0]
            + specular * m.albedo[1]
            + reflection_color * m.reflectivity
            + refraction_color * m.transparency;

        color
    } else {
        procedural_sky(*ray_direction)
    }
}



fn procedural_sky(dir: Vector3) -> Vector3 {
    let d = dir.normalized();
    let t = (d.y + 1.0) * 0.5; // map y [-1,1] → [0,1]

    let green = Vector3::new(0.1, 0.6, 0.2); // grass green
    let white = Vector3::new(1.0, 1.0, 1.0); // horizon haze
    let blue = Vector3::new(0.3, 0.5, 1.0);  // sky blue

    if t < 0.54 {
        // Bottom → fade green to white
        let k = t / 0.55;
        green * (1.0 - k) + white * k
    } else if t < 0.55 {
        // Around horizon → mostly white
        white
    } else if t < 0.8 {
        // Fade white to blue
        let k = (t - 0.55) / (0.25);
        white * (1.0 - k) + blue * k
    } else {
        // Upper sky → solid blue
        blue
    }
}

// pub fn render(framebuffer: &mut Framebuffer, objects: &[&dyn RayIntersect]) {
pub fn render(framebuffer: &mut Framebuffer, objects: &[&dyn RayIntersect], camera: &Camera, texture_manager: &TextureManager) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    let light = Light {
                position: Vector3::new(2.0, 4.0, -2.0),
                color: Vector3::new(1.0, 1.0, 1.0),
                intensity: 1.0,
            };

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();
            let rotated_direction = camera.basis_change(&ray_direction);

            let ray_color = cast_ray(&camera.eye, &rotated_direction, objects, &light, 0, texture_manager);
            let pixel_color = Color::new(
                (ray_color.x.clamp(0.0, 1.0) * 255.0) as u8,
                (ray_color.y.clamp(0.0, 1.0) * 255.0) as u8,
                (ray_color.z.clamp(0.0, 1.0) * 255.0) as u8,
                255,
            );
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
    framebuffer.set_background_color(Color::new(201, 201, 201, 255));

    let mut texture_manager = TextureManager::new();
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/brick.png");


    let purple_matte = Material {
        diffuse: Color::new(160, 110, 230, 255),
        specular: 32.0,
        reflectivity: 0.1,
        transparency: 0.0,
        refractive_index: 1.0,
        albedo: [0.8, 0.2],
        texture_path: Some("assets/brick.png".to_string())
    };

    let mirror = Material {
        diffuse: Color::WHITE,
        specular: 1000.0,
        reflectivity: 1.0,
        transparency: 0.0,
        refractive_index: 1.0,
        albedo: [0.0, 1.0],
        texture_path: Some("algo".to_string())
    };

    let glass = Material {
        diffuse: Color::WHITE,
        specular: 125.0,
        reflectivity: 0.1,
        transparency: 0.9,
        refractive_index: 1.5,
        albedo: [0.1, 0.9],
        texture_path: Some("algo".to_string())
    };


    let cube = Cube {
        center: Vector3::new(1.0, 0.0, -4.0),
        half_size: Vector3::new(1.0, 1.0, 1.0),
        rot_x: 20f32.to_radians(),
        rot_y: (-30f32).to_radians(),
        material: purple_matte,
    };
    
    let cube2 = Cube {
        center: Vector3::new(2.0, -3.0, -5.0),
        half_size: Vector3::new(1.0, 1.0, 1.0),
        rot_x: 20f32.to_radians(),
        rot_y: (-30f32).to_radians(),
        material: glass,
    };

    let cube3 = Cube {
        center: Vector3::new(5.0, 2.0, -5.0),
        half_size: Vector3::new(1.0, 1.0, 1.0),
        rot_x: 20f32.to_radians(),
        rot_y: (-30f32).to_radians(),
        material: mirror,
    };


    let objects_vec: Vec<&dyn RayIntersect> = vec![&cube,&cube2,&cube3];
    let objects_slice: &[&dyn RayIntersect] = &objects_vec;

     let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 10.0),  // eye
        Vector3::new(0.0, 0.0, 0.0),  // center
        Vector3::new(0.0, 1.0, 0.0),  // up
    );
    let rotation_speed = PI / 50.0;

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

        render(&mut framebuffer, objects_slice, &camera, &texture_manager);

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
