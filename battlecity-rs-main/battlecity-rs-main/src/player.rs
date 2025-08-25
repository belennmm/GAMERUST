use piston::Key;

use crate::{
    constants::{
        PLAYER_LIVES, PLAYER_MAX_ARMOR, PLAYER_MAX_HEALTH, PLAYER_SHOT_INTERVAL,
        PLAYER_SPAWN_ARMOR, PLAYER_SPAWN_HEALTH, TANK_1_TILES,
    },
    render::GameRenderObject,
    transform::LookDirection,
};

pub struct Player{
    id: u32,
    // position of jugador 
    position: [[i32; 2]; 2],

    lives: u32,
    health: u32,
    armor: u32,
    max_health: u32 ,
    max_armor: u32 ,

    kills: u32,

    is_alive: bool,
    spawn: [i32; 2],
    spawn_health: u32,
    spawn_armor: u32,

    last_shot_dt: f64,
    shot_interval: f64,

    
    movement_controls: [Key; 4],
    movement_controls_state: [bool; 4] ,
    fire_control: Key,
    fire_control_state: bool,

    direction: LookDirection,
    spawn_direction: LookDirection,

    tiles: [[f64; 4]; 8],
}

impl GameRenderObject for Player {
    fn is_visible(&self) -> bool { self.health > 0 }

    fn get_frame(&self) -> &[f64; 4] {
        self.frame_uv()
    }

    fn get_position(&self) -> &[i32; 2] { &self.position[0] }
    fn get_previous_position(&self) -> &[i32; 2] { &self.position[1] }
}

impl Player{
    pub fn new(

        id: u32,
        spawn: [i32; 2],
        spawn_direction: LookDirection,
        movement_controls: [Key; 4],
        fire_control: Key,
    ) -> Player {
       
        Player{
            id,
            position: [spawn, spawn],
            lives: PLAYER_LIVES,
            health: PLAYER_SPAWN_HEALTH ,
            armor: 0,
            max_health: PLAYER_MAX_HEALTH,
            max_armor: PLAYER_MAX_ARMOR ,
            kills: 0,
            is_alive: true ,
            spawn,
            spawn_health: PLAYER_SPAWN_HEALTH,
            spawn_armor: PLAYER_SPAWN_ARMOR,
            last_shot_dt: 0.0,
            shot_interval: PLAYER_SHOT_INTERVAL,
            movement_controls,
            movement_controls_state: [false; 4],
            fire_control,
            fire_control_state: false,
            direction: spawn_direction,
            spawn_direction,
            tiles: TANK_1_TILES,
        }
    }

    pub fn set_tiles(mut self, tiles: [[f64; 4]; 8]) -> Self {
        self.tiles = tiles;
        self
    }

    pub fn on_frame(&mut self, dt: f64) {
        self.last_shot_dt += dt;
    }

    pub fn shoot(&mut self) -> bool {
        if self.is_reloading() { return false; }
        self.last_shot_dt = 0.0;
        true
    }

    pub fn is_reloading(&self) -> bool { self.last_shot_dt < self.shot_interval }

    pub fn get_direction(&self) -> &LookDirection { &self.direction }
    pub fn set_direction(&mut self, direction: LookDirection) { self.direction = direction; }

    pub fn set_position(&mut self, position: [i32; 2]) {
        self.position[1] = self.position[0];
        self.position[0] = position;
    }
    pub fn get_position(&self) -> [i32; 2] { self.position[0] }

    pub fn get_id(&self) -> u32 { self.id }

    pub fn damage(&mut self) -> bool {
        if self.armor > 0 { self.armor -= 1; } else { self.health -= 1; }
        if self.health == 0 { self.is_alive = false; self.lives -= 1; }
        !self.is_alive
    }

    pub fn get_kills(&self) -> u32 { self.kills }
    pub fn inc_kill_count(&mut self) { self.kills += 1; }
    pub fn get_is_alive(&self) -> bool { self.is_alive }
    pub fn get_lives(&self) -> u32 { self.lives }
    pub fn can_respawn(&self) -> bool { self.lives > 0 }

    pub fn respawn(&mut self) {
        self.position[0] = self.spawn;
        self.position[1] = self.spawn;
        self.health = self.spawn_health;
        self.armor = self.spawn_armor;
        self.direction = self.spawn_direction;
        self.is_alive = true;
    }

    pub fn reset(&mut self) {
        self.lives = PLAYER_LIVES;
        self.kills = 0;
        self.respawn();
    }

    // -------- helpers ------------------------------------------------------- :)

    pub fn movement_key(&self, idx: usize) -> Key { self.movement_controls[idx] }
    pub fn set_move_pressed(&mut self, idx: usize, pressed: bool) {
        
        self.movement_controls_state[idx] = pressed;
    }

    pub fn fire_key(&self) -> Key { self.fire_control }
    pub fn set_fire_pressed(&mut self, pressed: bool){
        self.fire_control_state = pressed ;
    }

    pub fn get_pressed_direction(&self) -> Option<LookDirection>{
        self.movement_controls_state
            .iter()
            .position(|&b| b)
             .map(|i| match i {
                0 => LookDirection::Up,
                1 => LookDirection::Right,
                2 => LookDirection::Down,
                3 => LookDirection::Left,
                _ => unreachable!() ,
            })
    }

    pub fn get_is_fire_pressed(&self) -> bool {self.fire_control_state}

    /// UVs for armadura
    pub fn frame_uv(&self) -> &[f64; 4]{
        let shift = if self.armor > 0 { 4 } else { 0 };
        
        match self.direction{
            LookDirection::Up    => &self.tiles[0 + shift],
            LookDirection::Right => &self.tiles[1 + shift],
            LookDirection::Down  => &self.tiles[2 + shift],
            LookDirection::Left  => &self.tiles[3 + shift],
        }
    }

    // pinston vemos si lo uso
    pub fn on_press(&mut self, key: Key) {
        if let Some(i) = self.movement_controls.iter().position(|k| key.eq(k)) {
            self.movement_controls_state[i] = true; }
        if key.eq(&self.fire_control) { self.fire_control_state = true; }
    }

    pub fn on_release(&mut self, key: Key) {
        if let Some(i) = self.movement_controls.iter().position(|k| key.eq(k)) {
            self.movement_controls_state[i] = false;}
        if key.eq(&self.fire_control) { self.fire_control_state = false; }
    }

    pub(crate) fn add_armor(&mut self) -> bool {
    self.armor = std::cmp::min(self.max_armor, self.armor + 1);
    true
}

pub(crate) fn add_health(&mut self) -> bool {
    self.health = std::cmp::min(self.max_health, self.health + 1);
    true
}

pub fn get_health(&self) -> u32 { self.health }
pub fn get_armor(&self)  -> u32 { self.armor  }

}
