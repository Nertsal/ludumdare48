use super::*;

#[derive(Debug, Clone)]
pub struct TreeRoots {
    pub roots: HashMap<Id, Root>,
}

impl TreeRoots {
    pub fn new() -> Self {
        Self {
            roots: HashMap::new(),
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
    Head { direction: Direction },
}

type Direction = Vec2<f32>;

impl Model {
    pub fn update_roots(&mut self) {
        let ids: Vec<Id> = self.tree_roots.roots.keys().copied().collect();
        for id in ids {
            let mut root = self.tree_roots.roots.get(&id).unwrap().clone();
            self.update_root(&mut root);
            *self.tree_roots.roots.get_mut(&id).unwrap() = root;
        }
    }

    fn update_root(&mut self, root: &mut Root) {
        if let &mut RootType::Head { direction } = &mut root.root_type {
            if global_rng().gen::<f32>() <= self.rules.split_chance {
                self.split_root(root)
            } else {
                self.grow_root_random(root, direction);
            }
        }
    }

    fn grow_root_random(&mut self, root: &mut Root, direction: Direction) {
        let next_pos =
            root.position + direction * self.fixed_delta_time * self.rules.root_growth_speed;
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

    fn split_root(&mut self, root: &mut Root) {
        let left_pos = root.position
            + get_random_dir(f32::PI / 2.0, f32::PI)
                * self.fixed_delta_time
                * self.rules.root_growth_speed;
        let right_pos = root.position
            + get_random_dir(0.0, f32::PI / 2.0)
                * self.fixed_delta_time
                * self.rules.root_growth_speed;
        let id = self.new_root(Root {
            position: root.position,
            parent_root: root.parent_root,
            root_type: RootType::Node,
        });
        self.new_root(Root {
            position: left_pos,
            parent_root: Some(id),
            root_type: RootType::Head {
                direction: get_random_dir(0.0, f32::PI),
            },
        });
        root.position = right_pos;
        root.parent_root = Some(id);
    }
}

fn get_random_dir(min_angle: f32, max_angle: f32) -> Direction {
    let angle = global_rng().gen_range(min_angle, max_angle);
    let (y, x) = angle.sin_cos();
    vec2(x, y)
}
