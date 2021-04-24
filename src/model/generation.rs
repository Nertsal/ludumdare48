use super::*;

impl Model {
    pub fn generate_area(&mut self, area: Area) {
        for y in area.y_min..=area.y_max {
            for x in area.x_min..=area.x_max {
                let position = vec2(x, y);
                let noise_value = self.noise.get(position.map(|x| x as f32));
                self.tiles
                    .insert(position, self.tile_from_noise_value(noise_value));
            }
        }
    }

    fn tile_from_noise_value(&self, noise_value: f32) -> Tile {
        if noise_value <= self.rules.stone_frequency {
            Tile::Stone
        } else {
            Tile::Dirt
        }
    }
}
