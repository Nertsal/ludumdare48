use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {}

#[derive(Debug)]
pub struct Model {
    config: Config,
}

impl Model {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    pub fn update(&mut self, delta_time: f32) {}
    pub fn handle_event(&mut self, event: &geng::Event) {}
}
