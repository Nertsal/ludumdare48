use super::*;

#[derive(Debug, Clone)]
pub struct TreeRoots {
    pub roots: HashMap<Id, Root>,
    pub attractors: Vec<Attractor>,
}

impl TreeRoots {
    pub fn new() -> Self {
        Self {
            roots: HashMap::new(),
            attractors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Root {
    pub position: Vec2<f32>,
    pub parent_root: Option<Id>,
    pub root_type: RootType,
}

#[derive(Debug, Clone)]
pub enum RootType {
    Node,
    Final,
    Consumer { position: Position },
    Head { velocity: Velocity },
}

#[derive(Debug, Clone)]
pub struct Attractor {
    pub position: Vec2<f32>,
    root: Id,
}

type Velocity = Vec2<f32>;

impl Model {
    pub fn update_roots(&mut self) {
        let ids: Vec<Id> = self.tree_roots.roots.keys().copied().collect();
        for id in ids {
            if let Some(root) = self.tree_roots.roots.get(&id) {
                let mut root = root.clone();
                self.update_root(&mut root, id);
                *self.tree_roots.roots.get_mut(&id).unwrap() = root;
            }
        }
        self.split_roots = false;
    }

    fn update_root(&mut self, root: &mut Root, root_id: Id) {
        match &mut root.root_type {
            RootType::Head { velocity } => {
                if let Some((index, attractor)) = self
                    .tree_roots
                    .attractors
                    .iter()
                    .enumerate()
                    .find(|(_, attractor)| attractor.root == root_id)
                {
                    if attractor.position.y < root.position.y {
                        self.tree_roots.attractors.remove(index);
                    } else {
                        let direction = (attractor.position - root.position).normalize();
                        *velocity = (*velocity
                            + direction / self.rules.root_inertia * self.fixed_delta_time)
                            .clamp(self.rules.root_growth_speed);
                    }
                }

                if self.split_roots {
                    self.split_root(root);
                } else {
                    let velocity = *velocity;
                    self.grow_root(root, velocity);
                }
                for (&other_id, other) in &self.tree_roots.roots {
                    if other_id != root_id
                        && Some(other_id) != root.parent_root
                        && (other.position - root.position).len()
                            <= self.fixed_delta_time * self.rules.root_growth_speed / 2.0
                    {
                        root.root_type = RootType::Final;
                        return;
                    }
                }

                if root.position.x.abs() > self.rules.chamber_width as f32 {
                    root.root_type = RootType::Final;
                    return;
                }

                let position = get_tile_pos(root.position);
                if let Some(tile) = self.tiles.get(&position) {
                    match tile {
                        Tile::Stone => {
                            root.root_type = RootType::Final;
                        }
                        Tile::Mineral { .. } => {
                            root.root_type = RootType::Consumer { position };
                        }
                        _ => (),
                    }
                }
            }
            RootType::Consumer { position } => {
                let consume_limit = self.rules.mineral_consume_speed * self.fixed_delta_time;
                let consumed = self.consume(&mut HashSet::new(), *position, consume_limit);
                if consumed == 0.0 {
                    root.root_type = RootType::Final;
                }
            }
            _ => (),
        }
    }

    fn consume(
        &mut self,
        consumed_pos: &mut HashSet<Position>,
        position: Position,
        mut consume_limit: f32,
    ) -> f32 {
        if consumed_pos.contains(&position) {
            return 0.0;
        };
        if let Some(Tile::Mineral { minerals }) = self.tiles.get_mut(&position) {
            let consume = minerals.min(consume_limit);
            self.minerals += consume;
            *minerals -= consume;
            consume_limit -= consume;
            consumed_pos.insert(position);
            let consume_neighbours = if consume_limit > 0.0 {
                let mut consumed = 0.0;
                for neighbour in get_neighbours(position) {
                    let consume = self.consume(consumed_pos, neighbour, consume_limit);
                    consumed += consume;
                    consume_limit -= consume;
                    if consume_limit <= 0.0 {
                        break;
                    }
                }
                consumed
            } else {
                0.0
            };
            consume + consume_neighbours
        } else {
            0.0
        }
    }

    fn grow_root(&mut self, root: &mut Root, velocity: Velocity) {
        let next_pos = root.position + velocity * self.fixed_delta_time;
        let id = self.new_root(Root {
            position: root.position,
            parent_root: root.parent_root,
            root_type: RootType::Node,
        });
        root.position = next_pos;
        root.parent_root = Some(id);
    }

    pub fn new_root(&mut self, root: Root) -> Id {
        let id = self.id_generator.gen();
        self.tree_roots.roots.insert(id, root);
        id
    }

    pub fn spawn_attractor(&mut self, position: Vec2<f32>) {
        if let Some((&closest_id, _)) = self
            .tree_roots
            .roots
            .iter()
            .filter_map(|(id, root)| {
                if let RootType::Head { .. } = root.root_type {
                    if root.position.y < position.y {
                        let distance = (root.position - position).len();
                        return Some((id, distance));
                    }
                }
                None
            })
            .min_by(|(_, da), (_, db)| da.partial_cmp(&db).unwrap())
        {
            self.tree_roots.attractors.push(Attractor {
                position,
                root: closest_id,
            })
        }
    }

    pub fn split_root(&mut self, root: &mut Root) {
        let left_dir = get_random_dir(f32::PI * 2.0 / 3.0, f32::PI * 5.0 / 6.0);
        let left_pos = root.position + left_dir * self.fixed_delta_time;
        let right_dir = get_random_dir(f32::PI / 6.0, f32::PI / 3.0);
        let right_pos = root.position + right_dir * self.fixed_delta_time;
        let id = self.new_root(Root {
            position: root.position,
            parent_root: root.parent_root,
            root_type: RootType::Node,
        });
        self.new_root(Root {
            position: left_pos,
            parent_root: Some(id),
            root_type: RootType::Head {
                velocity: left_dir * self.rules.root_growth_speed,
            },
        });
        root.position = right_pos;
        root.parent_root = Some(id);
        root.root_type = RootType::Head {
            velocity: right_dir * self.rules.root_growth_speed,
        };
    }
}

fn get_random_dir(min_angle: f32, max_angle: f32) -> Velocity {
    let angle = global_rng().gen_range(min_angle, max_angle);
    let (y, x) = angle.sin_cos();
    vec2(x, y)
}

fn get_tile_pos(pos: Vec2<f32>) -> Position {
    pos.map(|x| x.floor() as i32)
}

fn get_neighbours(pos: Position) -> impl Iterator<Item = Position> {
    ((pos.x - 1)..=(pos.x + 1)).flat_map(move |x| {
        ((pos.y - 1)..=(pos.y + 1))
            .filter(move |&y| y != 0 || x != 0)
            .map(move |y| vec2(x, y))
    })
}
