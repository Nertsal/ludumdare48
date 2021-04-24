use super::*;

mod generation;
mod multi_noise;
mod root;
mod rules;

use multi_noise::*;
use root::*;
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
        model.fill_area(model.get_area(0, 20), Tile::Dirt);
        model.set_tile(
            vec2(0, 0),
            Tile::Root(Root {
                parent_root: None,
                root_type: RootType::Head {
                    update_timer: model.rules.root_growth_time,
                },
            }),
        );
        model.generate(0, 100);
        model
    }
    pub fn update(&mut self, delta_time: f32) {
        self.update_roots(delta_time);
    }
    pub fn handle_event(&mut self, event: &geng::Event) {}
    fn generate(&mut self, depth_start: i32, depth_end: i32) {
        self.generate_area(self.get_area(depth_start, depth_end));
    }
}

type Position = Vec2<i32>;
type Area = AABB<i32>;

#[derive(Debug, Clone)]
pub enum Tile {
    Dirt,
    Stone,
    Root(Root),
}

impl Tile {
    pub fn new_root_head(parent_root: Option<Position>, update_timer: f32) -> Self {
        Self::Root(Root {
            parent_root,
            root_type: RootType::Head { update_timer },
        })
    }
}
