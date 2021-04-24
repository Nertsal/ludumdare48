use super::*;

mod generation;
mod id;
mod multi_noise;
mod root;
mod rules;

use id::*;
use multi_noise::*;
use root::*;
use rules::*;

pub struct Model {
    pub tiles: HashMap<Position, Tile>,
    pub tree_roots: TreeRoots,
    delta_time: f32,
    fixed_delta_time: f32,
    rules: Rules,
    noise: MultiNoise,
    id_generator: IdGenerator,
}

impl Model {
    pub fn new() -> Self {
        let mut model = Self {
            tiles: HashMap::new(),
            tree_roots: TreeRoots::new(),
            fixed_delta_time: 1.0 / 20.0,
            delta_time: 0.0,
            rules: Rules::default(),
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
            id_generator: IdGenerator::new(),
        };
        model.new_root(Root {
            position: vec2(0.0, 0.0),
            parent_root: None,
            root_type: RootType::Head {
                velocity: vec2(0.0, model.rules.root_growth_speed),
            },
        });
        model.fill_area(model.get_area(0, 20), Tile::Dirt);
        model.generate(0, 100);
        model
    }
    pub fn update(&mut self, delta_time: f32) {
        self.delta_time += delta_time;
        if self.delta_time >= self.fixed_delta_time {
            self.delta_time -= self.fixed_delta_time;
            self.update_roots();
        }
    }
    pub fn handle_event(&mut self, event: &geng::Event) {}
    pub fn handle_message(&mut self, message: Message) {
        match message {
            Message::SpawnAttractor { pos } => {
                self.spawn_attractor(pos);
            }
        }
    }
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
}
