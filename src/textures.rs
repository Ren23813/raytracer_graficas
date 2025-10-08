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

    /// Muestra un texel dado (u,v). Aquí u,v pueden estar fuera de [0,1] — se envuelven (repeat).
    pub fn sample_uv(&self, path: &str, u: f32, v: f32) -> Vector3 {
        if let Some(cpu_texture) = self.cpu_textures.get(path) {
            // wrap (repetir) coordenadas u,v incluso si están fuera de [0,1]
            let mut u_wrapped = u - u.floor(); // fract, pero funciona con negativos
            let mut v_wrapped = v - v.floor();
            if u_wrapped < 0.0 { u_wrapped += 1.0; }
            if v_wrapped < 0.0 { v_wrapped += 1.0; }

            // convierte a índices de píxel usando floor
            let fx = (u_wrapped * cpu_texture.width as f32).floor() as i32;
            let fy = ((1.0 - v_wrapped) * cpu_texture.height as f32).floor() as i32; // invertir v si tu imagen tiene origen top-left

            // rem_euclid para asegurar índice positivo dentro de rango
            let tx = fx.rem_euclid(cpu_texture.width);
            let ty = fy.rem_euclid(cpu_texture.height);

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
}

impl Default for TextureManager {
    fn default() -> Self {
        TextureManager {
            cpu_textures: HashMap::new(),
            textures: HashMap::new(),
        }
    }
}
