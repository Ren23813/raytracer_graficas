use raylib::prelude::*;

pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
    pixel_data: Vec<Color>,
    overlays: Vec<(String, i32, i32, i32, Color)>,

    gpu_texture: Option<Texture2D>,
    pub dirty: bool,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32, background_color: Color) -> Self {
        let size = (width * height) as usize;
        let pixel_data = vec![background_color; size];
        let color_buffer = Image::gen_image_color(width, height, background_color);
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
            pixel_data,
            overlays: Vec::new(),
            gpu_texture: None,
            dirty: true, // la primera vez se debe crear la textura
        }
    }

    pub fn clear(&mut self) {
        self.pixel_data.fill(self.background_color);
        self.color_buffer = Image::gen_image_color(self.width, self.height, self.background_color);
        self.dirty = true;
    }

    pub fn set_pixel(&mut self, x: i32, y: i32) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            let index = (y * self.width + x) as usize;
            self.pixel_data[index] = self.current_color;
            Image::draw_pixel(&mut self.color_buffer, x as i32, y as i32, self.current_color);
            self.dirty = true; // marcamos que el contenido cambió y debemos recargar la textura antes de mostrar
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
        self.clear();
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    // draw_text ahora reemplaza overlays (no acumular)
    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, font_size: i32, color: Color) {
        self.overlays.clear();
        self.overlays.push((text.to_string(), x, y, font_size, color));
    }

    pub fn render_to_file(&self, file_path: &str) {
        Image::export_image(&self.color_buffer, file_path);
    }

    // swap_buffers: crea/recarga la textura SOLO si `dirty == true`, si no reutiliza la misma GPU texture
    pub fn swap_buffers(&mut self, window: &mut RaylibHandle, raylib_thread: &RaylibThread) {
        if self.gpu_texture.is_none() || self.dirty {
            // (re)crear la textura desde la imagen actual
            let texture = window
                .load_texture_from_image(raylib_thread, &self.color_buffer)
                .unwrap_or_else(|e| panic!("Failed to create texture from framebuffer: {}", e));
            // al asignar Some(texture) el Texture2D anterior (si existía) será droppeado y
            // dependiendo del binding liberará la textura GPU (Drop suele llamar UnloadTexture).
            self.gpu_texture = Some(texture);
            self.dirty = false;
        }

        // dibujar la textura cacheada + overlays
        {
            let mut d = window.begin_drawing(raylib_thread);
            if let Some(ref tex) = self.gpu_texture {
                d.draw_texture(tex, 0, 0, Color::WHITE);
            }
            for (text, x, y, font_size, color) in &self.overlays {
                d.draw_text(text, *x, *y, *font_size, *color);
            }
            // end drawing al salir del scope
        }

        // limpio overlays para siguiente frame (si quieres mantener, cambia esto)
        self.overlays.clear();
    }

    pub fn get_pixel_color(&self, x: i32, y: i32) -> Option<Color> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            let index = (y * self.width + x) as usize;
            Some(self.pixel_data[index])
        } else {
            None
        }
    }
}
