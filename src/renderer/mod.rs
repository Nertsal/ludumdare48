use super::*;

pub struct Renderer {
    geng: Rc<Geng>,
    scale: f32,
    screen_center: Vec2<f32>,
    current_depth: f32,
    target_depth: f32,
    tile_size: f32,
    root_width: f32,
    attractor_size: f32,
}

pub enum Message {
    SplitRoot,
    SpawnAttractor { pos: Vec2<f32> },
}

impl Renderer {
    pub fn new(geng: &Rc<Geng>) -> Self {
        Self {
            geng: geng.clone(),
            scale: 1.0,
            screen_center: vec2(0.0, 0.0),
            current_depth: 0.0,
            target_depth: 0.0,
            tile_size: 10.0,
            root_width: 5.0,
            attractor_size: 3.0,
        }
    }
    fn scale(&self) -> f32 {
        self.scale * self.tile_size
    }
    fn offset(&self) -> Vec2<f32> {
        vec2(0.0, -self.current_depth)
    }
    pub fn update(&mut self, delta_time: f32) {
        self.current_depth += (self.target_depth - self.current_depth) * delta_time * 2.0;
    }
    pub fn draw(&mut self, framebuffer: &mut ugli::Framebuffer, model: &model::Model) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        let screen_center = framebuffer.size().map(|x| (x as f32) / 2.0);
        self.screen_center = screen_center;

        self.target_depth = model
            .tree_roots
            .roots
            .values()
            .map(|root| root.position.y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        for (pos, tile) in model
            .tiles
            .iter()
            .filter(|(pos, _)| self.is_on_screen(pos.map(|x| x as f32)))
        {
            let color = match tile {
                Tile::Stone => Color::GRAY,
                Tile::Dirt => Color::rgb(0.5, 0.5, 0.0),
                Tile::Mineral => Color::CYAN,
            };
            let local_pos = self.world_to_camera(pos.map(|x| x as f32));
            self.geng.draw_2d().quad(
                framebuffer,
                AABB::from_corners(
                    local_pos - vec2(0.5, 0.5) * self.scale(),
                    local_pos + vec2(0.5, 0.5) * self.scale(),
                ),
                color,
            );
        }

        for root in model
            .tree_roots
            .roots
            .values()
            .filter(|root| self.is_on_screen(root.position))
        {
            let color = Color::rgb(0.2, 0.2, 0.0);
            let local_pos = self.world_to_camera(root.position);
            if let Some(parent) = root.parent_root {
                let parent_pos = model.tree_roots.roots[&parent].position;
                let parent_pos = self.world_to_camera(parent_pos);
                let vertices = [local_pos, parent_pos];
                self.geng.draw_2d().draw(
                    framebuffer,
                    &vertices,
                    color,
                    ugli::DrawMode::Lines {
                        line_width: self.root_width,
                    },
                )
            } else {
                self.geng.draw_2d().quad(
                    framebuffer,
                    AABB::from_corners(
                        local_pos - vec2(0.5, 0.5) * self.root_width,
                        local_pos + vec2(0.5, 0.5) * self.root_width,
                    ),
                    color,
                );
            }
        }

        for attractor in model
            .tree_roots
            .attractors
            .iter()
            .filter(|attractor| self.is_on_screen(attractor.position))
        {
            let color = Color::BLUE;
            let local_pos = self.world_to_camera(attractor.position);
            self.geng
                .draw_2d()
                .circle(framebuffer, local_pos, self.attractor_size, color);
        }
    }
    pub fn handle_event(&mut self, event: &geng::Event) -> Option<Message> {
        match event {
            geng::Event::MouseDown { position, button } => match button {
                geng::MouseButton::Left => Some(Message::SplitRoot),
                geng::MouseButton::Right => Some(Message::SpawnAttractor {
                    pos: self.camera_to_world(position.map(|x| x as f32)),
                }),
                _ => None,
            },
            _ => None,
        }
    }
    fn world_to_camera(&self, pos: Vec2<f32>) -> Vec2<f32> {
        let pos = vec2(pos.x, -pos.y);
        (pos - self.offset()) * self.scale() + self.screen_center
    }
    fn camera_to_world(&self, pos: Vec2<f32>) -> Vec2<f32> {
        let pos = (pos - self.screen_center) / self.scale() + self.offset();
        let pos = vec2(pos.x, -pos.y);
        pos
    }
    fn is_on_screen(&self, pos: Vec2<f32>) -> bool {
        let local_pos = self.world_to_camera(pos);
        local_pos.y >= 0.0 && local_pos.y <= self.screen_center.y * 2.0
    }
}
