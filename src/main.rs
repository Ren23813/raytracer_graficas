// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]

use raylib::prelude::*;
use std::f32::consts::PI;

mod framebuffer;
mod ray_intersect;
mod sphere;
mod material;

use framebuffer::Framebuffer;
use ray_intersect::RayIntersect;
use ray_intersect::HitInfo;
use sphere::Sphere;
use material::Material;

pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[Sphere],
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
        // Dirección de la luz (de arriba a la derecha)
        let light_dir = Vector3::new(-1.0, -1.0, -1.0).normalized();
        let diffuse_intensity = hit.normal.dot(-light_dir).max(0.0); // clamp to [0, 1]

        // Obtén el color difuso de la esfera que fue impactada
        let base_color = hit.object.material.diffuse;  // Usa el color difuso de la esfera
        let shaded = Vector3::new(base_color.r as f32, base_color.g as f32, base_color.b as f32) * diffuse_intensity;

        return Color::new(
            shaded.x as u8,
            shaded.y as u8,
            shaded.z as u8,
            255,
        );
    }

    // Fondo
    Color::new(4, 12, 36, 255)
}



pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere]) {
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
            let ray_origin = Vector3::new(0.0, 0.0, 0.0);

            let pixel_color = cast_ray(&ray_origin, &ray_direction, objects);

            framebuffer.set_current_color(pixel_color);
            framebuffer.set_pixel(x, y);
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer Example")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as i32, window_height as i32,Color::BLUE);

    framebuffer.set_background_color(Color::new(165, 221, 240, 255));

    let objects = [
    
    Sphere {
        center: Vector3::new(0.0, -1.0, -5.0),
        radius: 1.0,
        material: Material{diffuse:Color::SKYBLUE}
    },
    
    Sphere {
        center: Vector3::new(0.0, 0.3, -5.0),
        radius: 0.7,
        material: Material{diffuse:Color::SKYBLUE}
    },

    Sphere {
        center: Vector3::new(0.0, 1.2, -5.0),
        radius: 0.4,
        material: Material{diffuse:Color::SKYBLUE}
    },
    Sphere {
        center: Vector3::new(-0.1, 1.1, -4.0),
        radius: 0.05,
        material: Material{diffuse:Color::BLACK}
    },
    Sphere {
        center: Vector3::new(0.1, 1.1, -4.0),
        radius: 0.05,
        material: Material{diffuse:Color::BLACK}
    },
    Sphere {
        center: Vector3::new(0.0, 1.0, -4.0),
        radius: 0.05,
        material: Material{diffuse:Color::ORANGE}
    },
    ];


    while !window.window_should_close() {
        framebuffer.clear();

        render(&mut framebuffer, &objects);

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
