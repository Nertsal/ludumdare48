#[derive(Debug, Clone)]
pub struct Rules {
    pub root_growth_time: f32,
    pub chamber_width: usize,
    pub stone_frequency: f32,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            root_growth_time: 2.0,
            chamber_width: 20,
            stone_frequency: 0.2,
        }
    }
}
