use super::*;

mod client_view;
mod generation;
mod id;
mod multi_noise;
mod root;
mod rules;

pub use client_view::*;
use id::*;
use multi_noise::*;
use root::*;
use rules::*;

pub struct Model {
    pub tiles: HashMap<Position, Tile>,
    pub tree_roots: TreeRoots,
    delta_time: f32,
    fixed_delta_time: f32,
    pub rules: Rules,
    noises: [MultiNoise; 2],
    id_generator: IdGenerator,
    pub minerals: f32,
    split_root: Option<(Id, Vec2<f32>)>,
    client_view_update: ClientView,
    current_depth: f32,
    generation_depth: i32,
}

type Position = Vec2<i32>;
type Area = AABB<i32>;

#[derive(Debug, Clone)]
pub enum Tile {
    Dirt,
    Stone,
    Mineral { minerals: f32 },
}

impl Model {
    pub fn new() -> Self {
        let terrain_noise_properties = MultiNoiseProperties {
            min_value: 0.0,
            max_value: 1.0,
            scale: 20.0,
            octaves: 1,
            lacunarity: 1.0,
            persistance: 1.0,
        };
        let mut model = Self {
            tiles: HashMap::new(),
            tree_roots: TreeRoots::new(),
            fixed_delta_time: 1.0 / 20.0,
            delta_time: 0.0,
            rules: Rules::default(),
            noises: [
                MultiNoise::new(global_rng().gen(), &terrain_noise_properties),
                MultiNoise::new(
                    global_rng().gen(),
                    &MultiNoiseProperties {
                        scale: 5.0,
                        ..terrain_noise_properties
                    },
                ),
            ],
            id_generator: IdGenerator::new(),
            minerals: 0.0,
            split_root: None,
            client_view_update: ClientView::new(Rules::default()),
            current_depth: 0.0,
            generation_depth: 0,
        };
        model.reset();
        model
    }
    pub fn reset(&mut self) {
        self.tiles.clear();
        self.tree_roots = TreeRoots::new();
        for noise in &mut self.noises {
            noise.set_seed(global_rng().gen());
        }
        self.id_generator = IdGenerator::new();
        self.minerals = 10.0;
        self.split_root = None;
        self.generation_depth = 0;
        self.current_depth = 0.0;
        self.client_view_update = ClientView::new(self.rules.clone());

        self.new_root(Root {
            position: vec2(0.0, 0.0),
            parent_root: None,
            root_type: RootType::Head {
                velocity: vec2(0.0, self.rules.root_growth_speed),
            },
        });
        self.fill_area(self.get_area(0, 20), Tile::Dirt);
        self.generate();
    }
    pub fn update(&mut self, delta_time: f32) {
        self.delta_time += delta_time;
        if self.delta_time >= self.fixed_delta_time {
            self.delta_time -= self.fixed_delta_time;
            self.update_roots();

            self.current_depth = self
                .tree_roots
                .roots
                .values()
                .map(|root| root.position.y)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

            self.client_view_update.current_depth = self.current_depth;
            self.client_view_update.minerals = self.minerals;

            self.generate();
        }
    }
    pub fn handle_event(&mut self, event: &geng::Event) {}
    pub fn handle_message(&mut self, message: Message) {
        match message {
            Message::SpawnAttractor { pos } => {
                if self.try_spend(self.rules.attractor_cost) {
                    self.spawn_attractor(pos);
                }
            }
            Message::SplitRoot { pos } => {
                if self.can_spend(self.rules.split_cost) && self.split_towards(pos) {
                    self.try_spend(self.rules.split_cost);
                }
            }
        }
    }
    fn generate(&mut self) {
        if self.generation_depth - (self.current_depth as i32) < self.rules.generation_depth_min {
            self.remove_above(self.current_depth as i32 - self.rules.deletion_depth);
            self.generate_area(self.get_area(
                self.generation_depth,
                self.generation_depth + self.rules.generation_depth_max,
            ));
            self.generation_depth += self.rules.generation_depth_max;
        }
    }
    fn can_spend(&mut self, cost: f32) -> bool {
        self.minerals >= cost
    }
    fn try_spend(&mut self, cost: f32) -> bool {
        if self.can_spend(cost) {
            self.minerals -= cost;
            return true;
        }
        false
    }
}
