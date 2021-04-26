use super::*;
#[derive(Debug, Clone)]
pub struct ClientView {
    pub rules: Rules,
    pub current_depth: f32,
    pub minerals: f32,
    pub tiles: HashMap<Position, ViewEvent<Tile>>,
    pub roots: HashMap<Id, ViewEvent<Root>>,
    pub attractors: Vec<ViewEvent<Attractor>>,
}

impl Default for ClientView {
    fn default() -> Self {
        Self {
            rules: Rules::default(),
            current_depth: 0.0,
            minerals: 0.0,
            tiles: HashMap::new(),
            roots: HashMap::new(),
            attractors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ViewEvent<T: Debug + Clone> {
    Changed(T),
}

impl Model {
    pub fn get_client_view(&self) -> ClientView {
        ClientView {
            rules: self.rules.clone(),
            current_depth: self.current_depth,
            minerals: self.minerals,
            tiles: {
                let mut tiles = HashMap::with_capacity(self.tiles.len());
                for (&pos, tile) in &self.tiles {
                    tiles.insert(pos, ViewEvent::Changed(tile.clone()));
                }
                tiles
            },
            roots: {
                let mut roots = HashMap::with_capacity(self.tree_roots.roots.len());
                for (&id, root) in &self.tree_roots.roots {
                    roots.insert(id, ViewEvent::Changed(root.clone()));
                }
                roots
            },
            attractors: {
                let mut attractors = Vec::with_capacity(self.tree_roots.attractors.len());
                for attractor in &self.tree_roots.attractors {
                    attractors.push(ViewEvent::Changed(attractor.clone()));
                }
                attractors
            },
        }
    }
    pub fn get_client_view_update(&mut self) -> ClientView {
        ClientView {
            rules: self.rules.clone(),
            current_depth: self.current_depth,
            minerals: self.minerals,
            tiles: mem::take(&mut self.client_view_update.tiles),
            roots: mem::take(&mut self.client_view_update.roots),
            attractors: mem::take(&mut self.client_view_update.attractors),
        }
    }
}
