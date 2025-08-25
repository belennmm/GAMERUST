use crate::{render::GameRenderObject, constants::{BRICK_TILE, CONCRETE_TILE, EMPTY_FRAME_TILE, NET_TILE, WALL_HEIGHT}};
use glam::Vec3;
use crate::transform::tile_to_world;

pub fn wall_center_for(tile: [i32; 2]) -> Vec3 {
  
    tile_to_world(tile) + Vec3::new(0.0, WALL_HEIGHT * 0.5, 0.0)
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WallType {
    Brick,
    Concrete,
    Net,
    Empty,
    Border, 
}

pub struct Wall {
    variant: WallType,
    position: [i32; 2],
}

impl GameRenderObject for Wall {
    fn is_visible(&self) -> bool {
        self.variant != WallType::Empty
    }

    fn get_frame(&self) -> &[f64; 4] {
        self.get_frame()
    }

    fn get_position(&self) -> &[i32; 2] {
        &self.position
    }

    fn get_previous_position(&self) -> &[i32; 2] {
        &self.position
    }
}

impl Wall {
    pub fn new(position: [i32; 2]) -> Wall {
        Wall {
            variant: WallType::Empty,
            position,
        }
    }

    pub fn variant(&self) -> WallType {
        self.variant
    }

    pub fn brick(mut self) -> Self {
        self.variant = WallType::Brick;
        self
    }

    pub fn concrete(mut self) -> Self {
        self.variant = WallType::Concrete;
        self
    }

    pub fn net(mut self) -> Self {
        self.variant = WallType::Net;
        self
    }

    pub fn empty(mut self) -> Self {
        self.variant = WallType::Empty;
        self
    }

    pub fn damage(&mut self) {
        match self.variant {
            WallType::Brick => self.variant = WallType::Empty,
            WallType::Concrete => self.variant = WallType::Concrete,
            WallType::Net => self.variant = WallType::Net,
            WallType::Empty => self.variant = WallType::Empty,
            WallType::Border   => self.variant = WallType::Border,
        }
    }

    pub fn is_solid(&self) -> bool {
        match self.variant {
            WallType::Brick | WallType::Concrete => true,
            WallType::Net | WallType::Empty => false,
            WallType::Brick | WallType::Concrete | WallType::Border => true,    
        }
    }

    pub fn get_frame(&self) -> &[f64; 4] {
        match self.variant {
            WallType::Brick => &BRICK_TILE,
            WallType::Concrete => &CONCRETE_TILE,
            WallType::Net => &NET_TILE,
            WallType::Empty => &EMPTY_FRAME_TILE,
            WallType::Border   => &BRICK_TILE, 
        }
    }

    pub fn border(mut self) -> Self {   
        self.variant = WallType::Border;
        self
    }

}

pub fn generate_walls(column_count: u8, row_count: u8) -> Vec<Vec<Wall>> {
    let mut walls = vec![];

    for y in 0..row_count {
        let mut row = vec![];

        for x in 0..column_count {
            let rng = rand::random::<u8>() % 6;

            let w = match rng {
                 0 | 1 => Wall::new([x as i32, y as i32]).brick(),
                2     => Wall::new([x as i32, y as i32]).concrete(),
                3     => Wall::new([x as i32, y as i32]).net(),
                _     => Wall::new([x as i32, y as i32]).empty(),
            };
            row.push(w);
        }
        walls.push(row);
    }

     // para el border
    let w = column_count as usize;
    let h = row_count as usize;


    // top y 
    for x in 0..w {
        walls[0][x]       = Wall::new([x as i32, 0]).border();
        walls[h - 1][x]   = Wall::new([x as i32, (h - 1) as i32]).border();
    }
   
    for y in 0..h {
        walls[y][0]       = Wall::new([0, y as i32]).border();
        walls[y][w - 1]   = Wall::new([(w - 1) as i32, y as i32]).border();
    }

  
    [
        [0, 0],
        [row_count as i32 - 1, 0],
        [0, column_count as i32 - 1],
        [row_count as i32 - 1, column_count as i32 - 1],
    ]
    .iter()
    .for_each(|&[x, y]| {
        
    });

    walls
}

pub fn carve_safe_zone(walls: &mut Vec<Vec<Wall>>, center: [i32; 2], radius: i32) {
    if walls.is_empty() { return; }
    let h = walls.len() as i32;
    let w = walls[0].len() as i32;

    let [cx, cy] = center;
    for y in (cy - radius)..=(cy + radius) {
        if y < 0 || y >= h { continue; }
        for x in (cx - radius)..=(cx + radius) {
            if x < 0 || x >= w { continue; }
            walls[y as usize][x as usize] = Wall::new([x, y]).empty();
        }
    }
}

    