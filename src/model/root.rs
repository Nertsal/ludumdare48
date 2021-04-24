use super::*;

#[derive(Debug, Clone)]
pub struct Root {
    pub update_timer: f32,
    pub parent_root: Option<Position>,
    pub root_type: RootType,
}

#[derive(Debug, Clone)]
pub enum RootType {
    Node,
    Final,
    Head,
}

impl Model {
    pub fn update_roots(&mut self, delta_time: f32) {
        let positions: Vec<Position> = self
            .tiles
            .iter()
            .filter_map(|(pos, tile)| match tile {
                Tile::Root(root) => match &root.root_type {
                    RootType::Head => Some(pos.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        for pos in positions {
            if let Some(Tile::Root(root)) = self.tiles.get_mut(&pos) {
                root.update_timer -= delta_time;
                if root.update_timer <= 0.0 {
                    let direction = global_rng().gen_range(0, 3);
                    let next_pos = match direction {
                        0 => pos + vec2(-1, 1),
                        1 => pos + vec2(0, 1),
                        2 => pos + vec2(1, 1),
                        _ => unreachable!(),
                    };
                    let parent_root = root.parent_root;
                    self.set_tile(
                        next_pos,
                        Tile::Root(Root {
                            update_timer: self.rules.root_growth_time,
                            parent_root: Some(pos),
                            root_type: RootType::Head,
                        }),
                    );
                    self.set_tile(
                        pos,
                        Tile::Root(Root {
                            update_timer: 0.0,
                            parent_root,
                            root_type: RootType::Node,
                        }),
                    );
                }
            } else {
                unreachable!()
            }
        }
    }
}
