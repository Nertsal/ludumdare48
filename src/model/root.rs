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
    Head { velocity: Velocity },
}

#[derive(Debug, Clone)]
pub struct Attractor {
    pub position: Vec2<f32>,
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
    }

    fn update_root(&mut self, root: &mut Root, root_id: Id) {
        if let RootType::Head { velocity } = &mut root.root_type {
            let mut closest_attractor: Option<(f32, &Attractor)> = None;
            for attractor in &self.tree_roots.attractors {
                let distance = (root.position - attractor.position).len();
                if attractor.position.y > root.position.y
                    && (closest_attractor.is_none() || distance < closest_attractor.unwrap().0)
                {
                    closest_attractor = Some((distance, attractor));
                }
            }
            if let Some((_, attractor)) = closest_attractor {
                let direction = (attractor.position - root.position).normalize();
                *velocity = (*velocity
                    + direction / self.rules.root_inertia * self.fixed_delta_time)
                    .clamp(self.rules.root_growth_speed);
            }

            if global_rng().gen::<f32>() <= self.rules.split_chance * self.fixed_delta_time {
                self.split_root(root)
            } else {
                let velocity = velocity.clone();
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
        }

        if root.position.x.abs() > self.rules.chamber_width as f32 {
            root.root_type = RootType::Final;
            return;
        }

        if let Some(Tile::Stone) = self.tiles.get(&get_tile_pos(root.position)) {
            root.root_type = RootType::Final;
            return;
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

    pub fn spawn_attractor(&mut self, pos: Vec2<f32>) {
        self.tree_roots.attractors.push(Attractor { position: pos })
    }

    fn split_root(&mut self, root: &mut Root) {
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
