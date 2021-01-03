use crate::cell::Cell;
use std::cmp;
use std::collections::HashSet;

pub const WORLD_WIDTH: usize = 256;
pub const WORLD_HEIGHT: usize = 144;



pub const SAND_COLOR : [u8; 4] = [0xff, 0xe6, 0x66, 0xff];
pub const WATER_COLOR : [u8; 4] = [0x66, 0xb3, 0xff, 0xff];
pub const PLANT_COLOR : [u8; 4] = [0x99, 0xff, 0x66, 0xff];
pub const KELP_COLOR : [u8; 4] = [0x26, 0x4d, 0x00, 0xff];
pub const FISH_COLOR : [u8; 4] = [0xff, 0x8c, 0x19, 0xff];
pub const DECAY_COLOR : [u8; 4] = [0x1a, 0x00, 0x33, 0xff];
pub const BOTTOMFEEDER_COLOR : [u8; 4] = [0xcc, 0x66, 0xff, 0xff];
pub const ALGAE_COLOR : [u8; 4] = [0x19, 0xff, 0x66, 0xff];
pub const NITROGEN_COLOR : [u8; 4] = [0x33, 0x77, 0xff, 0xff];
pub const BUBBLE_COLOR : [u8; 4] = [0x56, 0xa3, 0xfe, 0xff];
pub const STONE_COLOR : [u8; 4] = [0x33, 0x33, 0x33, 0xff];
pub const WORM_COLOR : [u8; 4] = [0xBB, 0x44, 0x43, 0xff];

#[inline]
pub fn pos_to_idx(x: usize, y: usize) -> usize{
    return (y * WORLD_WIDTH) + x;
}

#[inline]
pub fn in_bounds(x: i32, y: i32) -> bool {
    x >= 0 && x <= (WORLD_WIDTH as i32 - 1) && y >=0 && y <= (WORLD_HEIGHT as i32 - 1)
}

#[inline]
fn clamp_to_bounds(xy: (i32, i32)) -> (usize, usize) {
    (cmp::min(cmp::max(xy.0, 0), WORLD_WIDTH as i32 -1) as usize, cmp::min(cmp::max(xy.1, 0), WORLD_HEIGHT as i32 - 1) as usize)
}

pub struct Cells {
    inner: Vec::<Cell>,
    lighting: Vec::<u8>,
    lighting_tmp: Vec::<u8>,
    current_offset: (i32, i32),
    ignored: HashSet<(i32, i32)>
}

impl Cells {
    fn new() -> Self {
        Self {
            inner: vec![Cell::Water; WORLD_WIDTH * WORLD_HEIGHT],
            lighting: vec![15u8; WORLD_WIDTH * WORLD_HEIGHT],
            lighting_tmp: vec![15u8; WORLD_WIDTH * WORLD_HEIGHT],
            current_offset: (0, 0),
            ignored: HashSet::new(),
        }
    }

    fn reset_ignored(&mut self) {
        self.ignored.clear();
    }

    pub fn ignore(&mut self, x: i32, y: i32) {
        self.ignored.insert((self.current_offset.0 + x, self.current_offset.1 + y));
    }

    fn ignored(&self, x: i32, y:i32) -> bool {
        self.ignored.get(&(self.current_offset.0 + x, self.current_offset.1 + y)).is_some()
    }

    fn set_offset(&mut self, x: i32, y: i32) {
        self.current_offset = (x, y);
    }

    pub fn current_pos(&self) -> (i32, i32) {
        self.current_offset
    }

    pub fn get(&self, dx: i32, dy: i32) -> Option<&Cell> {
        let (mut x, mut y) = (self.current_offset.0 + dx, self.current_offset.1 + dy);
        if !in_bounds(x, y) {
            None
        }
        else {
            let (nx, ny) = clamp_to_bounds((x, y));
            let idx = pos_to_idx(nx, ny);
            Some(&self.inner[idx])
        }        
    }

    pub fn set(&mut self, dx: i32, dy: i32, cell: Cell) {
        let (x, y) = clamp_to_bounds((self.current_offset.0 + dx, self.current_offset.1 + dy));
        let idx = pos_to_idx(x, y);
        self.inner[idx] = cell;
    }

    pub fn get_light(&mut self, dx: i32, dy: i32) -> u8 {
        let (x, y) = clamp_to_bounds((self.current_offset.0 + dx, self.current_offset.1 + dy));
        let idx = pos_to_idx(x, y);
        self.lighting[idx]
    }

    pub fn swap(&mut self, dx1: i32, dy1: i32, dx2: i32, dy2: i32) {
        let (x1, y1) = clamp_to_bounds((self.current_offset.0 + dx1, self.current_offset.1 + dy1));
        let (x2, y2) = clamp_to_bounds((self.current_offset.0 + dx2, self.current_offset.1 + dy2));
        self.inner.swap(
            pos_to_idx(x1, y1),
            pos_to_idx(x2, y2)
        );
    }

    pub fn calc_shadow(&mut self, smooth_lighting: bool) {
        // TODO use cos/sin for angled lighting

        for x in 0..WORLD_WIDTH {
            let mut s = 15u8;
            for y in 0..WORLD_HEIGHT {
                let i = pos_to_idx(x, y);
                self.lighting_tmp[i] = s;
                match self.inner[i] {
                    Cell::Stone => { s = s.saturating_sub(8) },
                    _ => {}
                }
            }
        }

        if smooth_lighting {
            for x in 0..WORLD_WIDTH {
                for y in 0..WORLD_HEIGHT {
                    let i = pos_to_idx(x, y);                
                    let mut v = vec!();
                    for dx in -3..3 {
                        for dy in -3..3 {
                            let (nx, ny) = clamp_to_bounds(((x as i32) + dx, (y as i32) + dy));
                            let i = pos_to_idx(nx, ny);
                            v.push(self.lighting_tmp[i] as u16)
                        }
                    }
                    self.lighting[i] = (v.iter().sum::<u16>() / v.len() as u16) as u8;
                }
            }
        }    
        else {
            std::mem::swap(&mut self.lighting, &mut self.lighting_tmp);
        }    
    }

    pub fn draw(&mut self, fb : &mut [u8], smooth_lighting: bool) {
        self.calc_shadow(smooth_lighting);
        for i in 0..self.inner.len() {
            let pixel = &mut fb[i * 4..(i * 4) + 4];
            let mut color = match self.inner[i] {
                Cell::Sand => {
                    SAND_COLOR
                },
                Cell::Plant {..} | Cell::Seed => {
                    PLANT_COLOR
                },
                Cell::KelpSeed | Cell::Kelp {..} | Cell::KelpLeaf => {
                    KELP_COLOR
                },
                Cell::Water => {
                    WATER_COLOR
                },
                Cell::Fish {..} | Cell::FishBody => {
                    FISH_COLOR
                },
                Cell::Decay => {
                    DECAY_COLOR
                },
                Cell::BottomFeeder => {
                    BOTTOMFEEDER_COLOR
                },
                Cell::Algae {..} => {
                    ALGAE_COLOR
                },
                Cell::Nitrogen {..} => {
                    NITROGEN_COLOR
                },
                Cell::Bubble | Cell::Fizzer => {
                    BUBBLE_COLOR
                },
                Cell::Stone => {
                    STONE_COLOR
                },
                Cell::Worm {..} | Cell::WormBody => {
                    WORM_COLOR
                }
            };
            let l = self.lighting[i];
            let s = (15 - l) * 8;
            color[0] = color[0].saturating_sub(s);
            color[1] = color[1].saturating_sub(s);
            color[2] = color[2].saturating_sub(s);
            pixel.copy_from_slice(&color);
        }
    }
}

pub struct World {
    cells: Cells,
    spawns: Vec::<((usize, usize), Cell)>
}

impl World {

    pub fn new() -> Self {
        let cells = Cells::new();

        Self {
            cells,
            spawns: vec!()
        }
    }

    pub fn spawn(&mut self, pos: (usize, usize), cell: Cell) {
        self.spawns.push((pos, cell));
    }
   

    pub fn update(&mut self) {
        self.cells.set_offset(0, 0);
        self.cells.reset_ignored();
        for ((x, y), c) in self.spawns.drain(0..).filter(|((x, y), _)| in_bounds(*x as i32, *y as i32)) {
            self.cells.set(x as i32, y as i32, c);
        }

        // TODO flip/flop which end of the x-axis to start on?
        for x in 0..WORLD_WIDTH{
            for y in (0..WORLD_HEIGHT).rev() {
                self.cells.set_offset(0, 0);
                if !self.cells.ignored(x as i32, y as i32) {
                    let cell = self.cells.get(x as i32, y as i32).unwrap().clone();
                    self.cells.set_offset(x as i32, y as i32);
                    cell.update(&mut self.cells);
                }                
            }
        }
    }

    pub fn draw(&mut self, fb : &mut [u8], smooth_lighting: bool) {
        self.cells.draw(fb, smooth_lighting);
    }
}