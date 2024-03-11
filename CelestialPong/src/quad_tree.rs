use macroquad::{color::{self, colors}, prelude::*};

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub half_width: f32,
    pub half_height: f32,

    left: f32,
    right: f32,
    up: f32,
    down: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Rect {
        return Rect {
            x,
            y,
            half_width: width / 2.,
            half_height: height / 2.,
            left: x-width/2.,
            right: x+width/2.,
            up: y-height/2.,
            down: x+height/2.,
        };
    }
    pub fn contains(&self, pos: Vec2) -> bool {
        return pos.x >= self.x - self.half_width
            && pos.x < self.x + self.half_width
            && pos.y >= self.y - self.half_height
            && pos.y < self.y + self.half_height;
    }

    pub fn overlap(&self, other : &Rect) -> bool {
        return !(self.right < other.left || self.left > other.right || self.up < other.down || self.down > other.up);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuadTreeEntry {
    position: Vec2,
    payload: usize,
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
    north_east: Option<Box<QuadTree>>,
    north_west: Option<Box<QuadTree>>,
    south_east: Option<Box<QuadTree>>,
    south_west: Option<Box<QuadTree>>,
}

impl QuadTree {
    pub fn new(area: Rect) -> QuadTree {
        return QuadTree {
            area,
            entries: [QuadTreeEntry::new(Vec2::ZERO, 0); QUADTREE_SIZE],
            number_of_entries: 0,
            north_east: Option::None,
            north_west: Option::None,
            south_east: Option::None,
            south_west: Option::None,
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
                    self.north_east = Some(Box::new(QuadTree::new(Rect::new(
                        self.area.x - self.area.half_width / 2.,
                        self.area.y - self.area.half_height / 2.,
                        self.area.half_width,
                        self.area.half_height,
                    ))));
                    self.north_west = Some(Box::new(QuadTree::new(Rect::new(
                        self.area.x + self.area.half_width / 2.,
                        self.area.y - self.area.half_height / 2.,
                        self.area.half_width,
                        self.area.half_height,
                    ))));
                    self.south_east = Some(Box::new(QuadTree::new(Rect::new(
                        self.area.x - self.area.half_width / 2.,
                        self.area.y + self.area.half_height / 2.,
                        self.area.half_width,
                        self.area.half_height,
                    ))));
                    self.south_west = Some(Box::new(QuadTree::new(Rect::new(
                        self.area.x + self.area.half_width / 2.,
                        self.area.y + self.area.half_height / 2.,
                        self.area.half_width,
                        self.area.half_height,
                    ))));
                }
            }
            true => {
                match self.north_east {
                    Some(ref mut qt) => qt.add(entry),
                    None => panic!("missing subnode!"),
                }

                match self.north_west {
                    Some(ref mut qt) => qt.add(entry),
                    None => panic!("missing subnode!"),
                }

                match self.south_east {
                    Some(ref mut qt) => qt.add(entry),
                    None => panic!("missing subnode!"),
                }

                match self.south_west {
                    Some(ref mut qt) => qt.add(entry),
                    None => panic!("missing subnode!"),
                }
            }
        }
    }

    pub fn debug_draw(&self) {
        let color = match self.area.overlap(&Rect::new(200., 200., 130., 130.)) {
            true=>color::BLUE,
            false=>color::RED
        };

        draw_rectangle_lines(
            self.area.x - self.area.half_width,
            self.area.y - self.area.half_height,
            self.area.half_width * 2.,
            self.area.half_height * 2.,
            2.,
            color,
        );

        match &self.north_east {
            Some(qt) => qt.debug_draw(),
            _ => {}
        }

        match &self.north_west {
            Some(qt) => qt.debug_draw(),
            _ => {}
        }

        match &self.south_east {
            Some(qt) => qt.debug_draw(),
            _ => {}
        }

        match &self.south_west {
            Some(qt) => qt.debug_draw(),
            _ => {}
        }
    }
}
