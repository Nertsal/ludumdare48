#[derive(Debug, Clone)]
pub struct Rules {
    pub root_growth_speed: f32,
    pub chamber_width: usize,
    pub stone_frequency: f32,
    pub split_chance: f32,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            root_growth_speed: 0.5,
            chamber_width: 20,
            stone_frequency: 0.2,
            split_chance: 0.0,
        }
    }
}
