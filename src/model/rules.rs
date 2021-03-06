#[derive(Debug, Clone)]
pub struct Rules {
    pub root_growth_speed: f32,
    pub chamber_width: usize,
    pub stone_frequency: f32,
    pub root_inertia: f32,
    pub mineral_frequency: f32,
    pub mineral_richness: f32,
    pub mineral_consume_speed: f32,
    pub split_cost: f32,
    pub attractor_cost: f32,
    pub generation_depth_max: i32,
    pub generation_depth_min: i32,
    pub deletion_depth: i32,
    pub root_size: f32,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            root_growth_speed: 2.0,
            chamber_width: 51,
            stone_frequency: 0.2,
            root_inertia: 1.0,
            mineral_frequency: 0.05,
            mineral_richness: 2.0,
            mineral_consume_speed: 1.0,
            split_cost: 1.0,
            attractor_cost: 2.0,
            generation_depth_max: 200,
            generation_depth_min: 100,
            deletion_depth: 20,
            root_size: 0.1,
        }
    }
}
