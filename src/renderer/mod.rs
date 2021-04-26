use super::*;

pub struct Renderer {
    geng: Rc<Geng>,
    scale: f32,
    texture_center: Vec2<f32>,
    current_depth: f32,
    target_depth: f32,
    tile_size: f32,
    root_width: f32,
    attractor_size: f32,
    texture_offset: f32,
    texture_buffer: usize,
    texture_size: Vec2<usize>,
    screen_size: Vec2<usize>,
    pub request_view: bool,
}

const DEFAULT_SCREEN_SIZE: Vec2<usize> = Vec2 { x: 1024, y: 768 };

pub enum Message {
    SplitRoot { pos: Vec2<f32> },
    SpawnAttractor { pos: Vec2<f32> },
}

impl Renderer {
    pub fn new(geng: &Rc<Geng>) -> Self {
        Self {
            geng: geng.clone(),
            scale: 1.0,
            texture_center: vec2(0.0, 0.0),
            current_depth: 0.0,
            target_depth: 0.0,
            tile_size: 10.0,
            root_width: 5.0,
            attractor_size: 3.0,
            texture_offset: 0.0,
            texture_buffer: 4,
            texture_size: vec2(0, 0),
            screen_size: vec2(0, 0),
            request_view: true,
        }
    }
    pub fn reset(&mut self, texture: &mut Option<ugli::Texture>) {
        self.target_depth = 0.0;
        self.current_depth = 0.0;
        self.texture_offset = 0.0;
        self.request_view = true;
        *texture = None;
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
    fn gen_texture(
        &mut self,
        texture: &mut Option<ugli::Texture>,
        // framebuffer: &ugli::Framebuffer,
    ) {
        // let size = framebuffer.size();
        let size = DEFAULT_SCREEN_SIZE;
        self.texture_size = vec2(size.x, size.y * self.texture_buffer);
        self.texture_center = size.map(|x| (x as f32) / 2.0);
        let mut temp_texture =
            ugli::Texture::new_uninitialized(self.geng.ugli(), self.texture_size);
        temp_texture.set_filter(ugli::Filter::Nearest);
        *texture = Some(temp_texture);
        self.request_view = true;
    }
    pub fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        view: &model::ClientView,
        texture: &mut Option<ugli::Texture>,
    ) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        self.screen_size = framebuffer.size();

        if texture.is_none() {
            self.gen_texture(texture); //, framebuffer);
        }

        {
            let mut framebuffer = ugli::Framebuffer::new_color(
                self.geng.ugli(),
                ugli::ColorAttachment::Texture(texture.as_mut().unwrap()),
            );
            self.draw_impl(&mut framebuffer, view);
        }
        let size = self.screen_size.map(|x| x as f32);
        let converter = vec2(
            size.x / DEFAULT_SCREEN_SIZE.x as f32,
            size.y / DEFAULT_SCREEN_SIZE.y as f32,
        );
        let texture_size = vec2(
            self.texture_size.x as f32 * converter.x,
            self.texture_size.y as f32 * converter.y,
        );
        let pos0 = self.texture_to_camera(vec2(0.0, 0.0)) - vec2(0.0, texture_size.y);
        self.geng.draw_2d().textured_quad(
            framebuffer,
            AABB::pos_size(pos0, texture_size),
            texture.as_ref().unwrap(),
            Color::WHITE,
        );

        let text = format!("Minerals: {}", view.minerals.floor());
        self.geng
            .default_font()
            .draw(framebuffer, &text, vec2(20.0, 20.0), 25.0, Color::WHITE);

        let text = format!("Score: {}", view.current_depth.floor());
        self.geng.default_font().draw_aligned(
            framebuffer,
            &text,
            vec2(size.x / 2.0, size.y - 50.0),
            0.5,
            25.0,
            Color::WHITE,
        );

        let overflow = self.texture_offset + (self.texture_buffer - 2) as f32 * size.y;
        if self.target_depth > overflow / self.scale() {
            self.texture_offset += (self.texture_buffer - 3) as f32 * size.y;
            self.gen_texture(texture); //, framebuffer);
        }
    }
    fn draw_impl(&mut self, framebuffer: &mut ugli::Framebuffer, view: &model::ClientView) {
        self.target_depth = view.current_depth;

        for (pos, tile) in &view.tiles {
            match tile {
                model::ViewEvent::Changed(tile) => {
                    let color = match tile {
                        Tile::Stone => Color::GRAY,
                        Tile::Dirt => Color::rgb(0.5, 0.5, 0.0),
                        Tile::Mineral { minerals } => Color::rgb(
                            0.1,
                            0.1,
                            (minerals / view.rules.mineral_richness).clamp(0.0, 1.0),
                        ),
                    };
                    let local_pos = self.world_to_texture(pos.map(|x| x as f32));
                    self.geng.draw_2d().quad(
                        framebuffer,
                        AABB::pos_size(local_pos, vec2(1.0, 1.0) * self.scale()),
                        color,
                    );
                }
            }
        }

        for root in view.roots.values() {
            match root {
                model::ViewEvent::Changed(root) => {
                    let color = Color::rgb(0.2, 0.2, 0.0);
                    let local_pos = self.world_to_texture(root.position);
                    if let Some((_, parent_pos)) = root.parent_root {
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
                            AABB::pos_size(
                                local_pos,
                                vec2(1.0, 1.0) * self.root_width * self.scale,
                            ),
                            color,
                        );
                    }
                }
            }
        }

        for attractor in &view.attractors {
            match attractor {
                model::ViewEvent::Changed(attractor) => {
                    let color = Color::BLUE;
                    let local_pos = self.world_to_texture(attractor.position);
                    self.geng
                        .draw_2d()
                        .circle(framebuffer, local_pos, self.attractor_size, color);
                }
            }
        }
    }
    pub fn handle_event(&mut self, event: &geng::Event) -> Option<Message> {
        match event {
            geng::Event::MouseDown { position, button } => match button {
                geng::MouseButton::Left => Some(Message::SplitRoot {
                    pos: self.camera_to_world(position.map(|x| x as f32)),
                }),
                geng::MouseButton::Right => Some(Message::SpawnAttractor {
                    pos: self.camera_to_world(position.map(|x| x as f32)),
                }),
                _ => None,
            },
            _ => None,
        }
    }
    fn world_to_texture(&self, pos: Vec2<f32>) -> Vec2<f32> {
        pos * self.scale() + vec2(self.texture_center.x, -self.texture_offset)
    }
    fn texture_to_camera(&self, pos: Vec2<f32>) -> Vec2<f32> {
        let offset = self.local_offset();
        let pos = pos
            + vec2(
                offset.x,
                self.texture_center.y * 1.5 - offset.y - self.texture_offset,
            );
        vec2(
            pos.x * self.screen_size.x as f32 / DEFAULT_SCREEN_SIZE.x as f32,
            pos.y * self.screen_size.y as f32 / DEFAULT_SCREEN_SIZE.y as f32,
        )
    }
    fn camera_to_world(&self, pos: Vec2<f32>) -> Vec2<f32> {
        let pos = vec2(
            pos.x * DEFAULT_SCREEN_SIZE.x as f32 / self.screen_size.x as f32,
            pos.y * DEFAULT_SCREEN_SIZE.y as f32 / self.screen_size.y as f32,
        );
        let pos = (pos - vec2(self.texture_center.x, self.texture_center.y * 1.5)) / self.scale()
            + self.offset();
        vec2(pos.x, -pos.y)
    }
}
