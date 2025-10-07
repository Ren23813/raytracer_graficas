use raylib::prelude::*;
use std::collections::HashMap;

pub struct CpuTexture {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<Vector3>, // Normalized RGB values
}

impl CpuTexture {
    pub fn from_image(image: &Image) -> Self {
        // cuidado con la API exacta de raylib-rs: aquí asumimos que
        // image.get_image_data() -> Vec<Color> (o ajusta según tu versión)
        let colors = image.get_image_data();
        let pixels = colors
            .iter()
            .map(|c| {
                Vector3::new(
                    c.r as f32 / 255.0,
                    c.g as f32 / 255.0,
                    c.b as f32 / 255.0,
                )
            })
            .collect();

        CpuTexture {
            width: image.width,
            height: image.height,
            pixels,
        }
    }
}

pub struct TextureManager {
    cpu_textures: HashMap<String, CpuTexture>,
    textures: HashMap<String, Texture2D>, // GPU textures para rendering
}

impl TextureManager {
    pub fn new() -> Self { Self::default() }

    pub fn load_texture(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &str,
    ) {
        if self.textures.contains_key(path) {
            return;
        }

        // Ajusta según la API de tu versión de raylib-rs si load_image devuelve Result
        let image = Image::load_image(path)
            .unwrap_or_else(|_| panic!("Failed to load image {}", path));

        let texture = rl
            .load_texture_from_image(thread, &image)
            .unwrap_or_else(|_| panic!("Failed to load texture {}", path));
            // si tu API devuelve Result: .unwrap_or_else(...)

        let cpu_texture = CpuTexture::from_image(&image);

        self.cpu_textures.insert(path.to_string(), cpu_texture);
        self.textures.insert(path.to_string(), texture);
    }

    /// Muestra un texel dado (u,v) en [0,1]
    pub fn sample_uv(&self, path: &str, u: f32, v: f32) -> Vector3 {
        if let Some(cpu_texture) = self.cpu_textures.get(path) {
            // mapear u,v en [0,1] a coordenadas de pixel
            let tx = (u * (cpu_texture.width as f32 - 1.0)).clamp(0.0, cpu_texture.width as f32 - 1.0) as i32;
            // v típicamente viene con origen en bottom o top; aquí asumimos v=0→bottom. Si tu atlas está invertido, cambia a (1.0-v).
            let ty = ((1.0 - v) * (cpu_texture.height as f32 - 1.0)).clamp(0.0, cpu_texture.height as f32 - 1.0) as i32;
            let index = (ty * cpu_texture.width + tx) as usize;
            if index < cpu_texture.pixels.len() {
                cpu_texture.pixels[index]
            } else {
                Vector3::one()
            }
        } else {
            Vector3::one()
        }
    }

    pub fn get_texture(&self, path: &str) -> Option<&Texture2D> {
        self.textures.get(path)
    }

    // mantiene default impl como antes
}

impl Default for TextureManager {
    fn default() -> Self {
        TextureManager {
            cpu_textures: HashMap::new(),
            textures: HashMap::new(),
        }
    }
}
