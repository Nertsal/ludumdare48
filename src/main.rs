use geng::prelude::*;

mod model;
mod renderer;

use model::*;
use renderer::*;

struct State {
    texture: Option<ugli::Texture>,
    model: Model,
    renderer: Renderer,
}

impl State {
    fn new(geng: &Rc<Geng>) -> Self {
        Self {
            texture: None,
            renderer: Renderer::new(geng),
            model: Model::new(),
        }
    }
}

impl geng::State for State {
    fn update(&mut self, delta_time: f64) {
        self.model.update(delta_time as f32);
        self.renderer.update(delta_time as f32);
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let view = if self.renderer.request_view {
            self.renderer.request_view = false;
            println!("Requesting full view");
            self.model.get_client_view()
        } else {
            self.model.get_client_view_update()
        };
        self.renderer.draw(framebuffer, &view, &mut self.texture);
    }
    fn handle_event(&mut self, event: geng::Event) {
        self.model.handle_event(&event);
        if let Some(message) = self.renderer.handle_event(&event) {
            self.model.handle_message(message);
        }
    }
}

fn main() {
    geng::setup_panic_handler();
    let geng = Rc::new(Geng::new(default()));
    let state = State::new(&geng);
    geng::run(geng, state);
}
