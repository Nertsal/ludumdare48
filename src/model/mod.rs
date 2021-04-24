use super::*;

mod generation;
mod multi_noise;
mod rules;

use multi_noise::*;
use rules::*;

pub struct Model {
    rules: Rules,
    pub tiles: HashMap<Position, Tile>,
    noise: MultiNoise,
}

impl Model {
    pub fn new() -> Self {
        let mut model = Self {
            rules: Rules::default(),
            tiles: HashMap::new(),
            noise: MultiNoise::new(
                global_rng().gen(),
                &MultiNoiseProperties {
                    min_value: 0.0,
                    max_value: 1.0,
                    scale: 20.0,
                    octaves: 1,
                    lacunarity: 1.0,
                    persistance: 1.0,
                },
            ),
        };
        model.generate(0, -100);
        model
    }
    pub fn update(&mut self, delta_time: f32) {}
    pub fn handle_event(&mut self, event: &geng::Event) {}
    fn generate(&mut self, depth_start: i32, depth_end: i32) {
        self.generate_area(AABB::from_corners(
            vec2(-(self.rules.chamber_width as i32), depth_start),
            vec2(self.rules.chamber_width as i32, depth_end),
        ))
    }
}

type Position = Vec2<i32>;
type Area = AABB<i32>;

#[derive(Debug, Clone, Copy)]
pub enum Tile {
    Dirt,
    Stone,
    Root,
}
