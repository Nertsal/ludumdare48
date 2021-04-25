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
    fn local_offset(&self) -> Vec2<f32> {
        self.offset() * self.scale()
    }
    pub fn update(&mut self, delta_time: f32) {
        self.current_depth += (self.target_depth - self.current_depth) * delta_time * 2.0;
    }
    pub fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        model: &model::Model,
        texture: &mut Option<ugli::Texture>,
    ) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        let screen_center = framebuffer.size().map(|x| (x as f32) / 2.0);
        self.screen_center = screen_center;
        let size = framebuffer.size();
        let texture_size = vec2(size.x, size.y * 2);

        if texture.is_none() {
            let mut temp_texture = ugli::Texture::new_uninitialized(self.geng.ugli(), texture_size);
            temp_texture.set_filter(ugli::Filter::Nearest);
            *texture = Some(temp_texture);
        }

        {
            let mut framebuffer = ugli::Framebuffer::new_color(
                self.geng.ugli(),
                ugli::ColorAttachment::Texture(texture.as_mut().unwrap()),
            );
            self.draw_impl(&mut framebuffer, model);
        }
        let size = framebuffer.size().map(|x| x as f32);
        let offset = self.local_offset();
        self.geng.draw_2d().textured_quad(
            framebuffer,
            AABB::pos_size(
                vec2(
                    offset.x,
                    size.y as f32 * 0.75 - offset.y - texture_size.y as f32,
                ),
                vec2(texture_size.x as f32, texture_size.y as f32),
            ),
            texture.as_ref().unwrap(),
            Color::WHITE,
        );

        let text = format!("Minerals: {}", model.minerals.floor());
        self.geng
            .default_font()
            .draw(framebuffer, &text, vec2(20.0, 20.0), 25.0, Color::WHITE);

        let text = format!("Score: {}", self.current_depth.floor());
        self.geng.default_font().draw_aligned(
            framebuffer,
            &text,
            vec2(self.screen_center.x, self.screen_center.y * 2.0 - 50.0),
            0.5,
            25.0,
            Color::WHITE,
        );
    }
    fn draw_impl(&mut self, framebuffer: &mut ugli::Framebuffer, model: &model::Model) {
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
                Tile::Mineral { minerals } => Color::rgb(
                    0.1,
                    0.1,
                    (minerals / model.rules.mineral_richness).clamp(0.0, 1.0),
                ),
            };
            let local_pos = self.world_to_texture(pos.map(|x| x as f32));
            self.geng.draw_2d().quad(
                framebuffer,
                AABB::from_corners(local_pos, local_pos + vec2(1.0, 1.0) * self.scale()),
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
            let local_pos = self.world_to_texture(root.position);
            if let Some(parent) = root.parent_root {
                let parent_pos = model.tree_roots.roots[&parent].position;
                let parent_pos = self.world_to_texture(parent_pos);
                let vertices = [local_pos, parent_pos];
                self.geng.draw_2d().draw(
                    framebuffer,
                    &vertices,
                    color,
                    ugli::DrawMode::Lines {
                        line_width: self.root_width * self.scale,
                    },
                )
            } else {
                self.geng.draw_2d().quad(
                    framebuffer,
                    AABB::from_corners(
                        local_pos,
                        local_pos + vec2(1.0, 1.0) * self.root_width * self.scale,
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
            let local_pos = self.world_to_texture(attractor.position);
            self.geng
                .draw_2d()
                .circle(framebuffer, local_pos, self.attractor_size, color);
        }

        self.geng.draw_2d().quad(
            framebuffer,
            AABB::pos_size(vec2(0.0, -50.0), vec2(100.0, 100.0)),
            Color::RED,
        );
    }
    pub fn handle_event(&mut self, event: &geng::Event) -> Option<Message> {
        match event {
            geng::Event::KeyDown { key: geng::Key::R } => {
                self.target_depth = 0.0;
                self.current_depth = 0.0;
                None
            }
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
    fn world_to_texture(&self, pos: Vec2<f32>) -> Vec2<f32> {
        pos * self.scale() + vec2(self.screen_center.x, 0.0)
    }
    fn camera_to_world(&self, pos: Vec2<f32>) -> Vec2<f32> {
        let pos = (pos - vec2(self.screen_center.x, 0.0)) / self.scale() + self.offset();
        let pos = vec2(pos.x, -pos.y);
        pos
    }
    fn is_on_screen(&self, pos: Vec2<f32>) -> bool {
        let local_pos = self.world_to_texture(pos) - self.local_offset();
        local_pos.y >= 0.0
            && local_pos.y <= self.screen_center.y * 2.0
            && local_pos.x >= 0.0
            && local_pos.x <= self.screen_center.x * 2.0
    }
}
