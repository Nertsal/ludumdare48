use super::*;

pub struct Renderer {
    geng: Rc<Geng>,
    scale: f32,
    current_depth: f32,
    tile_size: f32,
    root_width: f32,
}

impl Renderer {
    pub fn new(geng: &Rc<Geng>) -> Self {
        Self {
            geng: geng.clone(),
            scale: 1.0,
            current_depth: 0.0,
            tile_size: 10.0,
            root_width: 5.0,
        }
    }
    fn scale(&self) -> f32 {
        self.scale * self.tile_size
    }
    pub fn update(&mut self, delta_time: f32) {}
    pub fn draw(&mut self, framebuffer: &mut ugli::Framebuffer, model: &model::Model) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        let offset = vec2(0.0, self.current_depth);
        let screen_center = framebuffer.size().map(|x| (x as f32) / 2.0);

        for (pos, tile) in model.tiles.iter().filter(|(pos, _)| {
            is_on_screen(pos.map(|x| x as f32), offset, self.scale(), screen_center)
        }) {
            let color = match tile {
                Tile::Stone => Color::GRAY,
                Tile::Dirt => Color::rgb(0.5, 0.5, 0.0),
            };
            let local_pos =
                get_local_pos(pos.map(|x| x as f32), offset, self.scale(), screen_center);
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
            .filter(|root| is_on_screen(root.position, offset, self.scale(), screen_center))
        {
            let color = Color::rgb(0.2, 0.2, 0.0);
            let local_pos = get_local_pos(root.position, offset, self.scale(), screen_center);
            if let Some(parent) = root.parent_root {
                let parent_pos = model.tree_roots.roots[&parent].position;
                let parent_pos = get_local_pos(parent_pos, offset, self.scale(), screen_center);
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

        fn get_local_pos(
            pos: Vec2<f32>,
            offset: Vec2<f32>,
            scale: f32,
            screen_center: Vec2<f32>,
        ) -> Vec2<f32> {
            let pos = vec2(pos.x, -pos.y);
            (pos - offset) * scale + screen_center
        }

        fn is_on_screen(
            position: Vec2<f32>,
            offset: Vec2<f32>,
            scale: f32,
            screen_center: Vec2<f32>,
        ) -> bool {
            let y_max = offset.y + screen_center.y / scale;
            let y_min = offset.y - screen_center.y / scale;
            position.y >= y_min && position.y <= y_max
        }
    }
    pub fn handle_event(&mut self, event: &geng::Event) {}
}
