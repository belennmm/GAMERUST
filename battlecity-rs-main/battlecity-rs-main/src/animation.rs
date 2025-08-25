use glam::Vec3;
use crate::{render::GameRenderObject, constants::{EXPLOSION_FRAMES, SPAWN_FRAMES}};

#[derive(Copy, Clone, Debug)]
pub enum AnimMode { Once, Loop }

pub struct Animation {
 
    pub position: Vec3,          
    pub size: f32,              
    pub billboard: bool,    

   
    frames_f32: Vec<[f32; 4]>,
    frames_f64: Vec<[f64; 4]>,

    current_frame: usize,
    frame_duration: f32,
    frame_dt: f32,
    mode: AnimMode,

    pos2d: [i32; 2],
    prev2d: [i32; 2],
}

impl GameRenderObject for Animation{
    fn is_visible(&self) -> bool { true }
    fn get_frame(&self) -> &[f64; 4] {&self.frames_f64[self.current_frame]}
    fn get_position(&self) -> &[i32; 2] {&self.pos2d}
    fn get_previous_position(&self) -> &[i32; 2] {&self.prev2d}
}

impl Animation {
    pub fn new(position_2d: [i32; 2], frames_f64: Vec<[f64; 4]>, frame_duration: f32, mode: AnimMode) -> Self {
        let frames_f32 = frames_f64.iter().map(|[x,y,w,h]| [*x as f32,*y as f32,*w as f32,*h as f32]).collect();
        let position = Vec3::new(position_2d[0] as f32, position_2d[1] as f32, 0.0);
        
        Self{
            position ,
            size: crate::constants::SPRITE_SIZE,
            billboard: true,
            frames_f32 ,
            frames_f64,
            current_frame: 0,
            frame_duration ,
            frame_dt: 0.0,
            mode ,
            pos2d: position_2d,
            prev2d: position_2d,
        }
    }

    pub fn new_explosion(position_2d: [i32; 2]) -> Self {
        Self::new(position_2d, EXPLOSION_FRAMES.to_vec(), 0.10, AnimMode::Once)
    }

    pub fn new_spawn(position_2d: [i32; 2]) -> Self {
        Self::new(position_2d, SPAWN_FRAMES.to_vec(), 0.10, AnimMode::Loop) }

    pub fn on_frame(&mut self, dt: f32){
        self.frame_dt += dt;
        if self.frame_dt >= self.frame_duration {
            self.frame_dt = 0.0;
            match self.mode {
                AnimMode::Loop => {
                    self.current_frame = (self.current_frame + 1) % self.frames_f32.len();
                }
                AnimMode::Once => {
                    if self.current_frame + 1 < self.frames_f32.len() {
                        self.current_frame += 1;
                    }
                }
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.mode, AnimMode::Once) && self.current_frame == self.frames_f32.len() - 1
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.prev2d = self.pos2d;
        self.position = pos;
        self.pos2d = [pos.x.round() as i32, pos.z.round() as i32]; // keep grid-ish cache
    }

    pub fn current_frame_uv_pixels(&self) -> [f32; 4] {
        self.frames_f32[self.current_frame]
    }
}
