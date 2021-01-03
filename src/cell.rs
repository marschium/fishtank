use rand::prelude::*;
use crate::world::Cells;


#[inline]
fn once_every(n: u32) -> bool {
    thread_rng().gen_range(0..=n) == n
}

#[inline]
fn random_direction() -> (i32, i32) {
    let dir_choices : Vec::<(i32, i32)> = vec!(
        (0, 1),
        (0, -1),
        (1, 0),
        (1, 1),
        (1, -1),
        (-1, 0),
        (-1, 1),
        (-1, -1),
    );
    *dir_choices.choose(&mut thread_rng()).unwrap()
}

#[derive(Clone, PartialEq, Eq, new)]
pub struct AlgaeUpdate {
    #[new(value = "1")]
    e: i32
}

impl AlgaeUpdate {
    pub fn update_algae(mut self, cells: &mut Cells) {

        // die
        if self.e < 0 && once_every(320) {
            cells.set(0, 0, Cell::Decay);
            return;
        }

        if once_every(1280) {
            self.e -= 1;
        }

        // lose energy without light
        if once_every(320) && cells.get_light(0, 0) <= 8 {
            self.e -= 1
        }

        // reproduce
        if self.e > 0 && once_every(1280) {
            let (x, y) = random_direction();
            if cells.get(x, y) == Some(&Cell::Water) {
                self.e -= 1;
                if thread_rng().gen::<bool>() {
                    cells.set(x, y, Cell::new_algae());
                }
            }
        }

        // eat
        if once_every(10) {
            for (x, y) in vec!((0, 1), (0, -1), (1, 0), (1, 1), (1, -1), (-1, 0), (-1, 1), (-1, -1),) {
                if cells.get(x, y) == Some(&Cell::Nitrogen){
                    cells.set(x, y, Cell::Bubble);
                    self.e += 1;
                }
            }
        }

        // move
        let (mut nx, mut ny) = (0, 0);
        if once_every(360) {
            let (x, y) = random_direction(); // TODO drift according to water current
            if cells.get(x, y) == Some(&Cell::Water) {
                cells.set(0, 0, Cell::Water);
                nx = x;
                ny = y;
            }
        }

        cells.set(nx, ny, Cell::Algae { update: self });
        cells.ignore(nx, ny);
    }
}

#[derive(Clone, PartialEq, Eq, new)]
pub struct PlantUpdate {
    #[new(value = "1")]
    e: i32
}

impl PlantUpdate {
    pub fn update_plant(mut self, cells:&mut Cells) {
        if self.e < 0 && once_every(180) {
            match cells.get(0, -1) {
                Some(Cell::Plant {..}) => {
                    
                },
                _=> {
                    cells.set(0, 0, Cell::Decay);
                    return
                }
            }
        }

        // lose energy without light
        if once_every(320) && cells.get_light(0, 0) <= 8 {
            self.e -= 1
        }

        if self.e > 0 && once_every(180) {
            let x =  *vec!(-1, 0, 0, 0, 1).choose(&mut thread_rng()).unwrap();
            match cells.get(x, -1) {
                Some(Cell::Water) => {
                    self.e -= 1;
                    cells.set(x, -1, Cell::new_plant());
                    cells.ignore(x, -1);
                },
                _ => {}
            }
        }       
        cells.set(0, 0, Cell::Plant { update: self }); 
        cells.ignore(0, 0);
    }
}

#[derive(Clone, PartialEq, Eq, new)]
pub struct KelpUpdate {
    #[new(value = "1")]
    e: i32
}

impl KelpUpdate {
    pub fn update_kelp(mut self, cells: &mut Cells) {
        // die
        if self.e < 0 && once_every(320) {
            cells.set(0, 0, Cell::Decay);
            return;
        }

        // lose energy without light
        if once_every(320) && cells.get_light(0, 0) <= 8 {
            self.e -= 1
        }

        if once_every(60) && self.e > 0 {

            // stop growing randomly
            if once_every(20) && self.e > 0 {
                self.e -= 1;
            }
            else {
                if cells.get(0, -1) == Some(&Cell::Water) {
                    self.e -= 1;
                    cells.set(0, -1, Cell::new_kelp());
                }
                if cells.current_pos().1 % 2 == 0 {
                    cells.set(1, 0, Cell::KelpLeaf);
                    cells.set(2, 0, Cell::KelpLeaf);
                    cells.set(3, 0, Cell::KelpLeaf);
                    cells.set(4, 0, Cell::KelpLeaf);
                    cells.set(-1, -1, Cell::KelpLeaf);
                    cells.set(-2, -1, Cell::KelpLeaf);
                    cells.set(-3, -1, Cell::KelpLeaf);
                    cells.set(-4, -1, Cell::KelpLeaf);
                }
            }            
        }

        cells.set(0, 0, Cell::Kelp { update: self});
        cells.ignore(0, 0);
    }
}

#[derive(Clone, PartialEq, Eq, new)]
pub struct WormUpdate {
    #[new(value = "0")]
    dx: i32,
    #[new(value = "0")]
    dy: i32,
    #[new(value = "5")]
    e: i32,
    #[new(value = "vec!((-1,0), (-1, 0), (-1,0), (-1, 0), (-1,0), (-1, 0))")] // each piece is relative to the one before
    body: Vec::<(i32, i32)>
}

impl WormUpdate {
    
    fn update_worm(mut self, cells: &mut Cells) {

        // eat
        if once_every(2) {
            let (dx, dy) = random_direction();
            match cells.get(dx, dy) {
                Some(Cell::Algae{..}) | Some(Cell::Decay) => {
                    self.e += 1;
                    cells.set(dx, dy, Cell::Water);
                    self.body.push(self.body.last().unwrap_or(&(0, 0)).clone());
                },
                _ => {}
            }
        }

        // check energy
        if once_every(360) {
            self.e -= 1;
            if self.e <= 0 {
                let mut body_x = 0;
                let mut body_y = 0;
                for (x, y) in &self.body {
                    body_x += x;
                    body_y += y;
                    cells.set(body_x, body_y, Cell::Decay);
                }
                cells.set(0, 0, Cell::Decay);
                return;
            }
        }

        // swim
        if once_every(5) {
            if once_every(20) {
                // change direction
                let tmp = random_direction();
                self.dx = tmp.0;
                self.dy = tmp.1; 
            }

            match cells.get(self.dx, self.dy) {
                Some(Cell::Water) => { },
                None => {
                    self.dx = -self.dx;
                    self.dy = -self.dy;
                },
                _ => {
                    self.dx = 0;
                    self.dy = 0;
                }
            }
            
            cells.set(0, 0, Cell::Water);
            let mut body_x = 0;
            let mut body_y = 0;

            for (x, y) in &self.body {
                body_x += x;
                body_y += y;
                cells.set(body_x, body_y, Cell::Water);
            }

            body_x = 0;
            body_y = 0;
            for (x, y) in &self.body {
                cells.set(body_x, body_y, Cell::WormBody);
                body_x += x;
                body_y += y;
            }
            self.body.insert(0, (self.dx * -1, self.dy * -1));
            self.body.pop();
            cells.set(self.dx, self.dy, Cell::Worm { update: self});
        }
        else {            
            cells.set(0, 0, Cell::Worm { update: self});
        }
    }
}

#[derive(Clone, PartialEq, Eq, new)]
pub struct FishUpdate {
    #[new(value = "0")]
    dx: i32,
    #[new(value = "0")]
    dy: i32,
    #[new(value = "5")]
    e: i32,
    #[new(value = "vec!((-1, -1), (-1, 0), (-1, 1), (-2, 0), (-3, -1), (-3, 1))")] // absolute offsets
    body: Vec::<(i32, i32)>
}

impl FishUpdate {

    fn update_fish(mut self, cells: &mut Cells) {

        // eat
        if once_every(2) {
            let (dx, dy) = random_direction();
            match cells.get(dx, dy) {
                Some(Cell::Plant {..}) | Some(Cell::Algae{..}) | Some(Cell::Kelp {..}) | Some(Cell::KelpLeaf) => {
                    self.e += 1;
                    cells.set(dx, dy, Cell::Water);
                    if once_every(4) && cells.get(0, 1) == Some(&Cell::Water){
                        cells.set(0, 1, Cell::Decay);
                    }
                },
                Some(Cell::Worm { update: u }) => {
                    self.e += 2;
                    for (x , y) in u.body.clone() {
                        cells.set(dx + x, dy + y, Cell::Decay);
                    }
                }
                _ => {}
            }
        }

        // check energy
        if once_every(360) {
            self.e -= 1;
            if self.e <= 0 {                
                let o = if self.dx >= 0 { 1 } else { -1 };
                for (x, y) in self.body.iter() {            
                    cells.set(*x * o, *y * o, Cell::Decay);
                }
                cells.set(0, 0, Cell::Decay);
                return;
            }
        }

        // swim
        if once_every(10) {


            // clear
            {
                let o = if self.dx >= 0 { 1 } else { -1 };
                for (x, y) in self.body.iter() {            
                    cells.set(*x * o, *y * o, Cell::Water);
                }
                cells.set(0, 0, Cell::Water);   
            }
                     

            if once_every(10) {
                // change direction
                let tmp = random_direction();
                self.dx = tmp.0;
                self.dy = tmp.1; 
            }

            
        
            match cells.get(self.dx, self.dy) {
                Some(Cell::Water) => {
                    let o = if self.dx >= 0 { 1 } else { -1 };
                    let mut blocked = false;
                    for (x, y) in self.body.iter() {            
                        blocked |= cells.get(self.dx + (*x * o), self.dy + (*y * o)) != Some(&Cell::Water);
                    }

                    if blocked {
                        self.dx = 0;
                        self.dy = 0;
                    }
                 },
                None => {
                    self.dx = -self.dx;
                    self.dy = -self.dy;
                },
                _ => {
                    self.dx = 0;
                    self.dy = 0;
                }
            }

            // draw
            {
                let o = if self.dx >= 0 { 1 } else { -1 };
                for (x, y) in self.body.iter() {            
                    cells.set(self.dx + (*x * o), self.dy + (*y * o), Cell::FishBody);
                }
                cells.set(self.dx, self.dy, Cell::Fish { update: self });   
            }        
        } 
        else {            
            cells.set(0, 0, Cell::Fish { update: self }); 
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Cell {
    Water,
    Sand,
    Seed,
    Plant { update: PlantUpdate },
    Fish { update: FishUpdate },
    Decay,
    BottomFeeder,
    Algae { update: AlgaeUpdate },
    Nitrogen,
    Bubble,
    Stone,
    Fizzer,
    KelpSeed,
    Kelp { update: KelpUpdate },
    KelpLeaf,
    FishBody,
    Worm { update: WormUpdate},
    WormBody
}

impl Cell {
    pub fn new_fish() -> Self {
        Cell::Fish {
            update: FishUpdate::new()
        }
    }

    pub fn new_plant() -> Self {
        Cell::Plant {
            update: PlantUpdate::new()
        }
    }

    pub fn new_kelp() -> Self {
        Cell::Kelp {
            update: KelpUpdate::new()
        }
    }

    pub fn new_algae() -> Self {
        Cell::Algae {
            update: AlgaeUpdate::new()
        }
    }

    pub fn new_worm() -> Self {
        Cell::Worm {
            update: WormUpdate::new()
        }
    }

    pub fn update(self, cells: &mut Cells) {
        match self {
            Cell::Sand | Cell::Decay => {
                self.mv(1, cells);
            },
            Cell::Seed => {
                self.update_seed(Cell::new_plant(), cells);
                self.mv(1, cells);
            },
            Cell::Plant { update } => {
                update.update_plant(cells);
            }
            Cell::Fish { update } => {
                update.update_fish(cells);      
            }
            Cell::BottomFeeder => {
                self.update_bottomfeeder(cells);
            },
            Cell::Algae { update } => {
                update.update_algae(cells);
            },
            Cell::Nitrogen => {
                self.update_gas(20, cells);
            },
            Cell::Bubble => {
                self.update_gas(10, cells);
            },
            Cell::Fizzer => {
                self.update_fizzer(cells);
                self.mv(1, cells);
            },
            Cell::KelpSeed => {
                self.update_seed(Cell::new_kelp(), cells);
                self.mv(1, cells);
            },
            Cell::Kelp { update }=> {
                update.update_kelp(cells);
            },
            Cell::Worm { update }=> {
                update.update_worm(cells);
            },
            Cell::Water | Cell::Stone | Cell::KelpLeaf | Cell::FishBody | Cell::WormBody => {

            }
        }
    }

    fn mv(&self, yd: i32, cells: &mut Cells) {
        let (x, _) = random_direction();
        if cells.get(x, yd) == Some(&Cell::Water) {
            cells.swap(0, 0, x, yd);
            cells.ignore(x, yd);
        }
    }

    fn update_fizzer(&self, cells: &mut Cells) {
        if once_every(20) {
            if cells.get(0, -1) == Some(&Cell::Water) {
                cells.set(0, -1, Cell::Bubble);
            }
        }        
    }

    fn update_gas(&self, t: u32, cells: &mut Cells) {
        if !once_every(t) {
            return
        }

        let (x, _) = random_direction();
        match cells.get(x, -1) {
            Some(&Cell::Water) => {
                cells.swap(0, 0, x, -1);
                cells.ignore(x, -1);
            },
            None => {
                cells.set(0, 0, Cell::Water);
            },
            Some(&Cell::Stone) | Some(&Cell::Fish {..}) | Some(&Cell::Worm {..}) | Some(&Cell::FishBody) | Some(&Cell::WormBody) => {
            },
            _=> {
                match cells.get(x, -2) {
                    Some(&Cell::Water) => {
                        cells.swap(x, -1, x, -2);
                        cells.swap(0, 0, x, -1);
                        cells.ignore(x, -1);
                    },
                    _ => {}
                }
            }
        }
    }

    fn update_seed(&self, plant_cell: Cell, cells: &mut Cells) {
        match cells.get(0, 1) {
            Some(Cell::Sand) => {
                if once_every(2) {
                    cells.set(0, 0, plant_cell);
                }
                else {
                    cells.set(0, 0, Cell::Water);
                }
            },
            Some(Cell::Water) => {
            }
            _ => {
                cells.set(0, 0, Cell::Water);
            }
        }
    }

    fn update_bottomfeeder(&self, cells: &mut Cells) {
        if once_every(3) {
            let x =  *vec!(-1, 1).choose(&mut thread_rng()).unwrap();        
            if cells.get(x, 0) == Some(&Cell::Decay) {
                cells.set(x, 0, Cell::Nitrogen);
            }
            else if cells.get(x, 1) == Some(&Cell::Decay) {
                cells.set(x, 1, Cell::Nitrogen);
            }
            else if cells.get(x, -1) == Some(&Cell::Decay) {
                cells.set(x, -1, Cell::Nitrogen);
            }
        }  

        if !once_every(10) {
            return;
        }

        let x =  *vec!(-1, 1).choose(&mut thread_rng()).unwrap();  
        let mut fell = false;
        if cells.get(0, 1) == Some(&Cell::Water) {
            cells.swap(0, 0, 0, 1);
            fell = true;
        }
        else {
            if thread_rng().gen::<bool>() && cells.get(1, 1) == Some(&Cell::Water) {
                cells.swap(0, 0, 1, 1);
                fell = true;
            }
            else if cells.get(-1, 1) == Some(&Cell::Water) {
                cells.swap(0, 0, -1, 1);
                fell = true;
            }
        }

        if !fell {
            if cells.get(x, 0) == Some(&Cell::Water) {
                cells.swap(0, 0, x, 0);
                cells.ignore(x, 0);
            } 
            else if cells.get(x, 1) == Some(&Cell::Water) {
                cells.swap(0, 0, x, 1);
                cells.ignore(x, 1);
            } 
            else if cells.get(x, -1) == Some(&Cell::Water) {
                cells.swap(0, 0, x, -1);
                cells.ignore(x, -1);
            }
        }
        
    }
}

