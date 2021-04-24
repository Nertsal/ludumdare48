use super::*;

impl Model {
    pub fn get_area(&self, depth_start: i32, depth_end: i32) -> Area {
        AABB::from_corners(
            vec2(-(self.rules.chamber_width as i32), depth_start),
            vec2(self.rules.chamber_width as i32, depth_end),
        )
    }

    pub fn fill_area(&mut self, area: Area, tile: Tile) {
        for y in area.y_min..=area.y_max {
            for x in area.x_min..=area.x_max {
                let position = vec2(x, y);
                self.set_tile(position, tile.clone());
            }
        }
    }

    pub fn generate_area(&mut self, area: Area) {
        for y in area.y_min..=area.y_max {
            for x in area.x_min..=area.x_max {
                let position = vec2(x, y);
                let noise_value = self.noise.get(position.map(|x| x as f32));
                let tile = self.tile_from_noise_value(noise_value);
                self.try_set_tile(position, tile);
            }
        }
    }

    pub fn set_tile(&mut self, position: Position, tile: Tile) {
        self.tiles.insert(position, tile);
    }

    pub fn try_set_tile(&mut self, position: Position, tile: Tile) {
        self.tiles.entry(position).or_insert(tile);
    }

    fn tile_from_noise_value(&self, noise_value: f32) -> Tile {
        if noise_value <= self.rules.stone_frequency {
            Tile::Stone
        } else {
            Tile::Dirt
        }
    }
}
