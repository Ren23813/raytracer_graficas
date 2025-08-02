use raylib::prelude::*;

pub struct Framebuffer{
    pub width:i32,
    pub height:i32,
    pub color_buffer: Image,
    background_color:Color,
    current_color:Color,
    pixel_data: Vec<Color>,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32, background_color:Color) -> Self{
        let size = (width * height) as usize;
        let pixel_data = vec![background_color; size];
        let color_buffer = Image::gen_image_color(width,height,background_color);
        Framebuffer{
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
            pixel_data
        }
    }

    pub fn clear(&mut self){
        self.pixel_data.fill(self.background_color);
        self.color_buffer = Image::gen_image_color(self.width,self.height,self.background_color)
    }

    pub fn set_pixel(&mut self, x:i32, y:i32){
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            let index = (y * self.width + x) as usize;
            self.pixel_data[index] = self.current_color;
            Image::draw_pixel(&mut self.color_buffer, x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_background_color(&mut self, color:Color){
        self.background_color = color;
        self.clear();
    }

    pub fn set_current_color(&mut self, color:Color){
        self.current_color = color;
    }

    pub fn render_to_file(&self, file_path:&str){
        Image::export_image(&self.color_buffer, file_path);
    }

    pub fn swap_buffers(&self, window: &mut RaylibHandle, raylib_thread: &RaylibThread,){
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer){
            let mut renderer = window.begin_drawing(raylib_thread);
            renderer.draw_texture(&texture,0,0,Color::WHITE);
        }
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
