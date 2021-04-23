use super::*;

pub struct Renderer {
    geng: Rc<Geng>,
}

impl Renderer {
    pub fn new(geng: &Rc<Geng>) -> Self {
        Self { geng: geng.clone() }
    }
    pub fn update(&mut self, delta_time: f32) {}
    pub fn draw(&mut self, framebuffer: &mut ugli::Framebuffer, model: &model::Model) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
    }
    pub fn handle_event(&mut self, event: &geng::Event) {}
}
