// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]

use raylib::prelude::*;
use std::f32::consts::PI;
use rayon::prelude::*;
use std::sync::Arc;

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
    objects: &[&(dyn RayIntersect + Sync)],
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

fn map_uv_for_cube(local_point: &Vector3, local_normal: &Vector3, half_size: &Vector3) -> Option<(f32, f32)> {
    // local_point está en coordenadas locales (ej: x in [-hx, +hx])
    let p = *local_point;
    let hx = half_size.x;
    let hy = half_size.y;
    let hz = half_size.z;

    // evita dividir por cero
    if hx.abs() < 1e-6 || hy.abs() < 1e-6 || hz.abs() < 1e-6 {
        return None;
    }

    if local_normal.x.abs() > 0.9 {
        // cara izquierda/derecha: u = z, v = y
        let u = (p.z + hz) / (2.0 * hz); // map z from [-hz, hz] -> [0,1]
        let v = (p.y + hy) / (2.0 * hy); // map y from [-hy, hy] -> [0,1]
        Some((u, v))
    } else if local_normal.y.abs() > 0.9 {
        // cara top/bottom: u = x, v = z
        let u = (p.x + hx) / (2.0 * hx);
        let v = (p.z + hz) / (2.0 * hz);
        Some((u, v))
    } else if local_normal.z.abs() > 0.9 {
        // cara front/back: u = x, v = y
        let u = (p.x + hx) / (2.0 * hx);
        let v = (p.y + hy) / (2.0 * hy);
        Some((u, v))
    } else {
        None
    }
}




pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[&(dyn RayIntersect + Sync)],
    lights: &[Light],
    depth: u32,
    texture_manager: &TextureManager,
) -> Vector3 {
    if depth > 3 {
        return Vector3::new(0.1, 0.1, 0.2); // sky
    }

    // Buscar el hit más cercano
    let mut closest_hit: Option<HitInfo> = None;
    for object in objects {
        if let Some(hit) = object.ray_intersect(ray_origin, ray_direction) {
            if closest_hit.is_none() || hit.distance < closest_hit.as_ref().unwrap().distance {
                closest_hit = Some(hit);
            }
        }
    }

    if let Some(hit) = closest_hit {
        let m = hit.material;

        // color base desde material (y/o textura)
        let mut base_color = Vector3::new(
            m.diffuse.r as f32 / 255.0,
            m.diffuse.g as f32 / 255.0,
            m.diffuse.b as f32 / 255.0,
        );

        if let Some(texture_path) = &m.texture_path {
            if let Some((u_raw, v_raw)) = map_uv_for_cube(&hit.local_point, &hit.local_normal, &hit.local_half_size) {
                let u_scaled = u_raw * hit.texture_repeat.x;
                let v_scaled = v_raw * hit.texture_repeat.y;
                base_color = texture_manager.sample_uv(texture_path, u_scaled, v_scaled);
            }
        }

        // Ambient (luz suave general, evita que todo sea negro)
        let ambient = Vector3::new(0.06, 0.06, 0.06);

        // acumuladores de iluminación
        let mut total_diffuse = ambient * base_color; // start with ambient * base color
        let mut total_specular = Vector3::zero();

        // vista (dirección del ojo)
        let view_dir = (*ray_origin - hit.point).normalized();

        // recorrer todas las luces
        for light in lights.iter() {
            // vector hacia la luz y distancia
            let mut lvec = light.position - hit.point;
            let dist = lvec.length();
            if dist <= 0.0 { continue; }
            let light_dir = lvec / dist; // normalizado

            // test de sombra: si hay algo entre el punto y la luz, atenua
            let shadow_origin = hit.point + hit.normal * 5e-3; // mejor epsilon
            let in_shadow = intersects_any(&shadow_origin, &light_dir, objects, dist - 1e-3);

            // atenuación simple (ajustable)
            // usa k pequeño para que la luz alcance más
            let k = 0.02_f32;
            let attenuation = light.intensity / (1.0 + k * dist * dist);

            // si está en sombra: ponemos una fracción residual (para evitar negro absoluto)
            let shadow_factor = if in_shadow { 0.15 } else { 1.0 };

            // difuso (Lambert)
            let ndotl = hit.normal.dot(light_dir).max(0.0);
            total_diffuse += base_color * ndotl * attenuation * light.color * shadow_factor;

            // especular (Blinn-Phong)
            let half = (view_dir + light_dir).normalized();
            let ndoth = hit.normal.dot(half).max(0.0);
            let spec = ndoth.powf(m.specular);
            total_specular += light.color * spec * attenuation * shadow_factor;
        }

        // Reflection recursiva
        let mut reflection_color = Vector3::new(0.1, 0.1, 0.2);
        if m.reflectivity > 0.0 {
            let rdir = reflect(ray_direction, &hit.normal).normalized();
            let rorigin = hit.point + hit.normal * 1e-3;
            reflection_color = cast_ray(&rorigin, &rdir, objects, lights, depth + 1, texture_manager);
        }

        // Refraction recursiva
        let mut refraction_color = Vector3::zero();
        if m.transparency > 0.0 {
            let refr_dir = refract(ray_direction, &hit.normal, m.refractive_index);
            let refr_dir = refr_dir.normalized();
            let rorigin = hit.point - hit.normal * 1e-3;
            refraction_color = cast_ray(&rorigin, &refr_dir, objects, lights, depth + 1, texture_manager);
        }

        // Emisión del material (si tiene)
        let emitted = m.emissive * m.emission;

        // Composición final (clamp implícito en conversión a color)
        let color = total_diffuse * m.albedo[0]
            + total_specular * m.albedo[1]
            + reflection_color * m.reflectivity
            + refraction_color * m.transparency
            + emitted;

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

pub fn render(framebuffer: &mut Framebuffer, objects: &[&(dyn RayIntersect + Sync)], camera: &Camera, texture_manager: &TextureManager) {
    let width_i = framebuffer.width;
    let height_i = framebuffer.height;
    let width = width_i as f32;
    let height = height_i as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    let lights_vec = vec![
        Light { // exterior / sol
            position: Vector3::new(2.5, 5.0, -7.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 1.2,
        },
        Light { // antorcha: posición justo encima del bloque visible
            position: Vector3::new(-3.0, -2.0, 2.0), 
            color: Vector3::new(1.0, 0.72, 0.35),
            intensity: 6.0,
        },
    ];

    let lights_arc = Arc::new(lights_vec);

    // Iterador paralelo: para cada fila (y) en paralelo
    let pixels: Vec<(i32, i32, Color)> = (0..height_i)
        .into_par_iter()
        .flat_map(|y| {
            let lights_for_row = lights_arc.clone();
            (0..width_i).into_par_iter().map(move |x| {
                let screen_x = (2.0 * x as f32) / width - 1.0;
                let screen_y = -(2.0 * y as f32) / height + 1.0;
                let sx = screen_x * aspect_ratio * perspective_scale;
                let sy = screen_y * perspective_scale;
                let ray_direction = Vector3::new(sx, sy, -1.0).normalized();
                let rotated_direction = camera.basis_change(&ray_direction);

                // calculamos color pasando todas las luces (slice desde Arc)
                let ray_color = cast_ray(&camera.eye, &rotated_direction, objects, &*lights_for_row, 0, texture_manager);

                let pixel_color = Color::new(
                    (ray_color.x.clamp(0.0, 1.0) * 255.0) as u8,
                    (ray_color.y.clamp(0.0, 1.0) * 255.0) as u8,
                    (ray_color.z.clamp(0.0, 1.0) * 255.0) as u8,
                    255,
                );

                (x, y, pixel_color)
            })
        })
        .collect();

    for (x, y, color) in pixels {
        framebuffer.set_current_color(color);
        framebuffer.set_pixel(x, y);
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

    window.set_target_fps(60);
    let mut framebuffer = Framebuffer::new(window_width as i32, window_height as i32, Color::BLACK);
    framebuffer.set_background_color(Color::new(201, 201, 201, 255));

    let mut texture_manager = TextureManager::new();
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/blackstone.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/brick.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/glass.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/log_spruce.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/glowstone.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/water_flow.png");


    let brick = Material {
        diffuse: Color::new(180, 80, 60, 255),  
        specular: 16.0,                         
        reflectivity: 0.03,                    
        transparency: 0.0,                      
        refractive_index: 1.0,                 
        albedo: [0.9, 0.1],                     
        texture_path: Some("assets/brick.png".to_string()),
        emissive:Vector3::zero(),
        emission:0.0
    };

    let blackstone = Material {
        diffuse: Color::new(160, 110, 230, 255),
        specular: 32.0,
        reflectivity: 0.1,
        transparency: 0.0,
        refractive_index: 1.0,
        albedo: [0.8, 0.2],
        texture_path: Some("assets/blackstone.png".to_string()),
        emissive:Vector3::zero(),
        emission:0.0
    };


    // let mirror = Material {
    //     diffuse: Color::WHITE,
    //     specular: 1000.0,
    //     reflectivity: 1.0,
    //     transparency: 0.0,
    //     refractive_index: 1.0,
    //     albedo: [0.0, 1.0],
    //     texture_path: Some("algo".to_string())
    // };

    let glass = Material {
        diffuse: Color::WHITE,
        specular: 90.0,
        reflectivity: 0.15,
        transparency: 0.9,
        refractive_index: 1.5,
        albedo: [0.05, 0.95],
        texture_path: Some("assets/glass.png".to_string()),  
        emissive:Vector3::zero(),
        emission:0.0
    };


    let wood = Material {
        diffuse: Color::new(100, 70, 50, 255),  
        specular: 8.0,                          
        reflectivity: 0.02,                      
        transparency: 0.0,                       
        refractive_index: 1.0,                
        albedo: [0.9, 0.1],                     
        texture_path: Some("assets/log_spruce.png".to_string()),
        emissive:Vector3::zero(),
        emission:0.0
    };

    let water = Material {
        diffuse: Color::new(60, 130, 200, 255),
        specular: 80.0,                        
        reflectivity: 0.08,                   
        transparency: 0.75,                     
        refractive_index: 1.333,                
        albedo: [0.05, 0.95],                 
        texture_path: Some("assets/water_flow.png".to_string()),
        emissive:Vector3::zero(),
        emission:0.0
    };

    let glowstone = Material {
        diffuse: Color::WHITE,
        specular: 12.0,
        reflectivity: 0.0,
        transparency: 0.0,
        refractive_index: 1.0,
        albedo: [0.6, 0.4],
        texture_path: Some("assets/glowstone.png".to_string()),
        emissive: Vector3::new(1.0, 0.6, 0.2),
        emission: 1.5,
    };



    let cube = Cube::new(
        Vector3::new(0.0, 4.1, 0.0),
        Vector3::new(4.0, 1.0, 4.0),
        0f32.to_radians(),
        (0f32).to_radians(),
        brick,
    );
    
    let cube2 = Cube::new(
        Vector3::new(0.0, 0.0, 4.0),
        Vector3::new(4.0, 3.2, 0.5), 
        0f32.to_radians(),
        (0f32).to_radians(),
        glass,
    );

    // let cube3 = Cube::new(
    //     Vector3::new(5.0, 2.0, -5.0),
    //     Vector3::new(1.0, 1.0, 1.0),
    //     20f32.to_radians(),
    //     (-30f32).to_radians(),
    //     mirror,
    // );

    let cube4 = Cube::new(
        Vector3::new(0.0, -4.0, 0.0),
        Vector3::new(4.0, 1.0, 4.0),
        0f32.to_radians(),
        (0f32).to_radians(),
        blackstone,
    );

    let water1 = Cube::new(
        Vector3::new(3.0 ,  -2.5, -1.0),
        Vector3::new(1.0, 0.5,3.8), 
        0.0, 0.0,
        water,
    );


    let cube5 = Cube::new(
        Vector3::new(-5.0, 0.0, 0.0),
        Vector3::new(1.0, 4.0, 4.0),
        0f32.to_radians(),
        (0f32).to_radians(),
        wood.clone(),
    );

    let cube6 = Cube::new(
        Vector3::new(5.0, 0.0, 0.0),
        Vector3::new(1.0, 4.0, 4.0),
        0f32.to_radians(),
        (0f32).to_radians(),
        wood.clone(),
    );
 
    let torch_obj = Cube::new(
        Vector3::new(-3.0, -2.2, 2.0),
        Vector3::new(0.8, 0.8,0.8),
        0.0, 
        0.0,
        glowstone,
    );

    let objects_vec: Vec<&(dyn RayIntersect + Sync)> = vec![
        &cube as &(dyn RayIntersect + Sync),
        &cube2 as &(dyn RayIntersect + Sync),
        // &cube3 as &(dyn RayIntersect + Sync),
        &cube4 as &(dyn RayIntersect + Sync),
        &cube5 as &(dyn RayIntersect + Sync),
        &cube6 as &(dyn RayIntersect + Sync),
        &water1 as &(dyn RayIntersect + Sync),
        &torch_obj as &(dyn RayIntersect + Sync),
    ];
    let objects_slice: &[&(dyn RayIntersect + Sync)] = &objects_vec;

    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, -15.0),  // eye
        Vector3::new(0.0, 0.0, 0.0),  // center
        Vector3::new(0.0, 1.0, 0.0),  // up
    );
    let rotation_speed = PI / 50.0;

    let mut camera_moved = true;

    while !window.window_should_close() {

        // detectar entrada y mover cámara
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.orbit(rotation_speed, 0.0);
            camera_moved = true;
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.orbit(-rotation_speed, 0.0);
            camera_moved = true;
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            camera.orbit(0.0, -rotation_speed);
            camera_moved = true;
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.orbit(0.0, rotation_speed);
            camera_moved = true;
        }

        // Si la cámara se movió, re-renderiza (pesado).
        if camera_moved {
            // render pinta en framebuffer.color_buffer / pixel_data y marca framebuffer.dirty via set_pixel o al final explicitamente
            render(&mut framebuffer, objects_slice, &camera, &texture_manager);
            // aseguramos que framebuffer se marque sucio (por si render no llamó a set_pixel internamente)
            framebuffer.dirty = true;
            camera_moved = false;
        }

        // dibujar FPS — simple y rápido: lo ponemos como overlay para que swap_buffers lo pinte.
        let fps = window.get_fps();
        let text = format!("FPS: {}", fps);
        framebuffer.draw_text(&text, 8, 8, 20, Color::BLACK);

        // swap_buffers dibuja la textura cacheada (rápido si dirty == false)
        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
