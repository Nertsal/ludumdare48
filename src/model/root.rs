use super::*;

#[derive(Debug, Clone)]
pub struct Root {
    pub parent_root: Option<Position>,
    pub root_type: RootType,
}

#[derive(Debug, Clone)]
pub enum RootType {
    Node,
    Final,
    Head { update_timer: f32 },
}

impl Model {
    pub fn update_roots(&mut self, delta_time: f32) {
        let positions: Vec<Position> = self
            .tiles
            .iter()
            .filter_map(|(pos, tile)| match tile {
                Tile::Root(root) => match &root.root_type {
                    RootType::Head { .. } => Some(pos.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        for pos in positions {
            if let Some(Tile::Root(root)) = self.tiles.get(&pos) {
                let root = root.clone();
                if let Some(root) = self.update_root(delta_time, root, pos) {
                    self.set_tile(pos, Tile::Root(root));
                }
            } else {
                unreachable!()
            }
        }
    }

    fn update_root(&mut self, delta_time: f32, mut root: Root, root_pos: Position) -> Option<Root> {
        if let RootType::Head { update_timer } = &mut root.root_type {
            *update_timer -= delta_time;
            if *update_timer <= 0.0 {
                self.grow_root_random(root, root_pos);
                None
            } else {
                Some(root)
            }
        } else {
            unreachable!()
        }
    }

    fn grow_root_random(&mut self, root: Root, root_pos: Position) {
        let direction = global_rng().gen_range(0, 3);
        let next_pos = match direction {
            0 => root_pos + vec2(-1, 1),
            1 => root_pos + vec2(0, 1),
            2 => root_pos + vec2(1, 1),
            _ => unreachable!(),
        };
        self.set_tile(
            next_pos,
            Tile::Root(Root {
                parent_root: Some(root_pos),
                root_type: RootType::Head {
                    update_timer: self.rules.root_growth_time,
                },
            }),
        );
        self.set_tile(
            root_pos,
            Tile::Root(Root {
                parent_root: root.parent_root,
                root_type: RootType::Node,
            }),
        );
    }
}
