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
    pub parent_root: Option<(Id, Vec2<f32>)>,
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
        let roots = &self.tree_roots.roots;
        if self.split_root.is_some() {
            for attractor in &mut self.tree_roots.attractors {
                if let Some(closest_id) = Self::closest_root_id(roots, attractor.position) {
                    attractor.root = closest_id;
                }
            }
        }
        self.split_root = None;
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

                let velocity = *velocity;
                if let Some((id, target)) = self.split_root {
                    if id == root_id {
                        self.split_root(root, target);
                    }
                }
                self.grow_root(root, velocity);

                for (&other_id, other) in &self.tree_roots.roots {
                    if other_id != root_id
                        && Some(other_id) != root.parent_root.map(|(id, _)| id)
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

                self.client_view_update
                    .roots
                    .insert(root_id, ViewEvent::Changed(root.clone()));
            }
            RootType::Consumer { position } => {
                let consume_limit = self.rules.mineral_consume_speed * self.fixed_delta_time;
                let consumed = self.consume(&mut HashSet::new(), *position, consume_limit);
                if consumed == 0.0 {
                    root.root_type = RootType::Final;
                }

                self.client_view_update
                    .roots
                    .insert(root_id, ViewEvent::Changed(root.clone()));
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
        if let Some(tile) = self.tiles.get_mut(&position) {
            if let Tile::Mineral { minerals } = tile {
                let consume = minerals.min(consume_limit);
                self.minerals += consume;
                *minerals -= consume;
                consume_limit -= consume;
                consumed_pos.insert(position);

                self.client_view_update
                    .tiles
                    .insert(position, ViewEvent::Changed(tile.clone()));

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

                return consume + consume_neighbours;
            }
        }
        0.0
    }

    fn grow_root(&mut self, root: &mut Root, velocity: Velocity) {
        let next_pos = root.position + velocity * self.fixed_delta_time;
        let id = self.new_root(Root {
            position: root.position,
            parent_root: root.parent_root,
            root_type: RootType::Node,
        });
        root.parent_root = Some((id, root.position));
        root.position = next_pos;
    }

    pub fn new_root(&mut self, root: Root) -> Id {
        let id = self.id_generator.gen();
        self.client_view_update
            .roots
            .insert(id, ViewEvent::Changed(root.clone()));
        self.tree_roots.roots.insert(id, root);
        id
    }

    pub fn spawn_attractor(&mut self, position: Vec2<f32>) {
        if let Some(closest_id) = Self::closest_root_id(&self.tree_roots.roots, position) {
            let attractor = Attractor {
                position,
                root: closest_id,
            };
            self.client_view_update
                .attractors
                .push(ViewEvent::Changed(attractor.clone()));
            self.tree_roots.attractors.push(attractor);
        }
    }

    fn closest_root_id(roots: &HashMap<Id, Root>, position: Vec2<f32>) -> Option<Id> {
        roots
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
            .map(|(&id, _)| id)
    }

    pub fn split_towards(&mut self, position: Vec2<f32>) -> bool {
        if let Some(closest) = Self::closest_root_id(&self.tree_roots.roots, position) {
            self.split_root = Some((closest, position));
            return true;
        }
        false
    }

    pub fn split_root(&mut self, root: &mut Root, target: Vec2<f32>) {
        let (left_dir, right_dir) = if target.x > root.position.x {
            (
                get_random_dir(f32::PI * 2.0 / 3.0, f32::PI * 5.0 / 6.0),
                (target - root.position).normalize(),
            )
        } else {
            (
                (target - root.position).normalize(),
                get_random_dir(f32::PI / 6.0, f32::PI / 3.0),
            )
        };
        let left_pos = root.position + left_dir * self.fixed_delta_time;
        let right_pos = root.position + right_dir * self.fixed_delta_time;

        let id = self.new_root(Root {
            position: root.position,
            parent_root: root.parent_root,
            root_type: RootType::Node,
        });
        self.new_root(Root {
            position: left_pos,
            parent_root: Some((id, root.position)),
            root_type: RootType::Head {
                velocity: left_dir * self.rules.root_growth_speed,
            },
        });
        root.parent_root = Some((id, root.position));
        root.position = right_pos;
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
