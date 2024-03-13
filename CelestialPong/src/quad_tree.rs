use macroquad::{
    color::{self},
    prelude::*,
};

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub half_width: f32,
    pub half_height: f32,

    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Rect {
        return Rect {
            x,
            y,
            half_width: width / 2.,
            half_height: height / 2.,

            left: x - width / 2.,
            right: x + width / 2.,
            up: y - height / 2.,
            down: y + height / 2.,
        };
    }
    pub fn contains(&self, pos: Vec2) -> bool {
        return pos.x >= self.x - self.half_width
            && pos.x < self.x + self.half_width
            && pos.y >= self.y - self.half_height
            && pos.y < self.y + self.half_height;
    }

    pub fn overlap(&self, other: &Rect) -> bool {
        return !(self.right < other.left
            || self.left > other.right
            || self.up > other.down
            || self.down < other.up);
    }

    pub fn debug_draw(&self, thickness: f32, color: Color) {
        draw_rectangle_lines(
            self.x - self.half_width,
            self.y - self.half_height,
            self.half_width * 2.,
            self.half_height * 2.,
            thickness,
            color,
        );
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuadTreeEntry {
    pub position: Vec2,
    pub payload: usize,
}

impl QuadTreeEntry {
    pub fn new(position: Vec2, payload: usize) -> QuadTreeEntry {
        return QuadTreeEntry { position, payload };
    }
}

const QUADTREE_SIZE: usize = 1;

#[derive(Clone, Debug)]
pub struct QuadTree {
    entries: [QuadTreeEntry; QUADTREE_SIZE],
    number_of_entries: usize,
    area: Rect,
    sub_trees: Option<Box<[QuadTree; 4]>>,
}

impl QuadTree {
    pub fn new(area: Rect) -> QuadTree {
        return QuadTree {
            area,
            entries: [QuadTreeEntry::new(Vec2::ZERO, 0); QUADTREE_SIZE],
            number_of_entries: 0,
            sub_trees: Option::None,
        };
    }

    fn is_full(&self) -> bool {
        return self.number_of_entries >= QUADTREE_SIZE;
    }

    pub fn add(&mut self, entry: QuadTreeEntry) {
        if !self.area.contains(entry.position) {
            return;
        }

        match self.is_full() {
            false => {
                self.entries[self.number_of_entries] = entry;
                self.number_of_entries = self.number_of_entries + 1;
                if self.is_full() {
                    self.sub_trees = Some(Box::new([
                        QuadTree::new(Rect::new(
                            self.area.x - self.area.half_width / 2.,
                            self.area.y - self.area.half_height / 2.,
                            self.area.half_width,
                            self.area.half_height,
                        )),
                        QuadTree::new(Rect::new(
                            self.area.x + self.area.half_width / 2.,
                            self.area.y - self.area.half_height / 2.,
                            self.area.half_width,
                            self.area.half_height,
                        )),
                        QuadTree::new(Rect::new(
                            self.area.x - self.area.half_width / 2.,
                            self.area.y + self.area.half_height / 2.,
                            self.area.half_width,
                            self.area.half_height,
                        )),
                        QuadTree::new(Rect::new(
                            self.area.x + self.area.half_width / 2.,
                            self.area.y + self.area.half_height / 2.,
                            self.area.half_width,
                            self.area.half_height,
                        )),
                    ]));
                }
            }
            true => match self.sub_trees {
                Some(ref mut sub_nodes) => {
                    for node in sub_nodes.iter_mut() {
                        node.add(entry);
                    }
                }
                None => panic!("missing subnodes!"),
            },
        }
    }

    pub fn query_entries(&self, query: &Rect, result: &mut Vec<QuadTreeEntry>) {
        if !self.area.overlap(query) {
            return;
        }

        for entry in self.entries {
            if query.contains(entry.position) {
                result.push(entry);
            }
        }

        match self.sub_trees {
            Some(ref sub_nodes) => {
                for node in sub_nodes.iter() {
                    node.query_entries(query, result);
                }
            }
            None => {}
        }
    }

    pub fn debug_draw(&self) {
        let color = color::RED;

        self.area.debug_draw(2., color);

        match &self.sub_trees {
            Some(sub_nodes) => {
                for node in sub_nodes.iter() {
                    node.debug_draw();
                }
            }
            None => {}
        }
    }
}
