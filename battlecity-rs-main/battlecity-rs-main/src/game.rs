
use crate::render::GameRenderObject; 
use raylib::prelude::{KeyboardKey, RaylibHandle};
use piston::Key;
use crate::camera::GameCamera;
use crate::transform::tile_to_world;

use crate::constants::{COLUMN_COUNT, ROW_COUNT};

const EYE_HEIGHT_FP: f32 = 0.5;   
const NOSE_OFFSET:  f32 = -0.35; 
const LOOK_DIST:    f32 = 6.0;  
const CAM_SMOOTH:   f32 = 0.20;  


use crate::{
    animation::Animation,
    constants::*,
    pickup::{Pickup, PickupSpawnSystem, PickupType},
    player::Player,
    projectile::Projectile,
    transform::LookDirection,
    wall::{generate_walls, Wall, WallType},

};

fn is_in_bounds(x: i32, y: i32, column_count: u8, row_count: u8) -> bool {
    x >= 0 && x < column_count as i32 && y >= 0 && y < row_count as i32
}

pub struct Game {
    column_count: u8,
    row_count: u8,
    players: Vec<Player>,
    walls: Vec<Vec<Wall>>,
    pickups: Vec<Pickup>,
    pickup_spawn_systems: [PickupSpawnSystem; 2],
    max_pickups: usize,
    bullets: Vec<Projectile>,
    animations: Vec<Animation>,
    accumulated_time: f64,
    last_update: f64,
    update_interval: f64,
    camera: GameCamera,

    // botsito
    bot_brains: Vec<Option<BotBrain>>,

   
    state: GameState,   
    state_time: f32,    
}


impl Game {
    
    pub fn new(column_count: u8, row_count: u8) -> Game {
        let players = vec![
            Player::new(
                0,
                [0, 0],
                LookDirection::Down,
                [Key::Up, Key::Right, Key::Down, Key::Left],
                Key::Space,
            )
            .set_tiles(TANK_1_TILES),
            Player::new(
                1,
                [column_count as i32 - 1, row_count as i32 - 1],
                LookDirection::Up,
                [Key::W, Key::D, Key::S, Key::A],
                Key::X,
            )
            .set_tiles(TANK_3_TILES),
            Player::new(
                2,
                [0, row_count as i32 - 1],
                LookDirection::Up,
                [Key::T, Key::H, Key::G, Key::F],
                Key::B,
            )
            .set_tiles(TANK_2_TILES),
            Player::new(
                3,
                [column_count as i32 - 1, 0],
                LookDirection::Down,
                [Key::I, Key::L, Key::K, Key::J],
                Key::M,
            )
            .set_tiles(TANK_4_TILES),
        ];

        let mut walls = generate_walls(column_count, row_count);

      
        for p in &players {
            let [sx, sy] = p.get_position();
            if sy >= 0 && (sy as usize) < walls.len()
                && sx >= 0 && (sx as usize) < walls[0].len()
            {
                walls[sy as usize][sx as usize] = crate::wall::Wall::new([sx, sy]).empty();
            }
        }

      
        for p in &players {
            crate::wall::carve_safe_zone(&mut walls, p.get_position(), 2);
        }

        let bot_brains = (0..players.len())
        .map(|i| if i == 0 { None } else { Some(BotBrain::default()) })
        .collect::<Vec<_>>();





        Game {
            column_count,
            row_count,
            players,
            bot_brains,
            walls,
            

            pickup_spawn_systems: [
                PickupSpawnSystem::new(PickupType::Armor, ARMOR_SPAWN_TIME),
                PickupSpawnSystem::new(PickupType::Health, HEALTH_SPAWN_TIME),
            ],

            max_pickups: MAX_SPAWNED_PICKUPS,
            pickups: vec![],
            bullets: vec![],
            animations: vec![],
            last_update: 0.0,
            accumulated_time: 0.0,
            update_interval: GAME_TICK_INTERVAL,
            camera: GameCamera::default(),

             state: GameState::Menu,
            state_time: 0.0,

            
        }
    }


    pub fn new_default() -> Game {
        Game::new(COLUMN_COUNT, ROW_COUNT)
    }

   pub fn poll_input_raylib(&mut self, rl: &raylib::prelude::RaylibHandle) {
        use raylib::prelude::KeyboardKey;

        fn map_key(k: piston::Key) -> KeyboardKey {
            use piston::Key::*;
            use KeyboardKey::*;
            match k {
                Up => KEY_UP, Right => KEY_RIGHT, Down => KEY_DOWN, Left => KEY_LEFT,
                W => KEY_W, A => KEY_A, S => KEY_S, D => KEY_D,
                T => KEY_T, H => KEY_H, G => KEY_G, F => KEY_F,
                I => KEY_I, L => KEY_L, K => KEY_K, J => KEY_J,
                X => KEY_X, B => KEY_B, M => KEY_M,
                Space => KEY_SPACE,
                _ => KEY_NULL,
            }
        }

        if let Some(p) = self.players.get_mut(0) {
            for i in 0..4 {
                let key = map_key(p.movement_key(i));
                p.set_move_pressed(i, key != KeyboardKey::KEY_NULL && rl.is_key_down(key));
            }
            let fire_key = map_key(p.fire_key());
            p.set_fire_pressed(fire_key != KeyboardKey::KEY_NULL && rl.is_key_down(fire_key));
        }
    }



    pub fn update_dt(&mut self, dt: f32) {
        // advance timers
        self.accumulated_time += dt as f64;

      
        self.animations.retain_mut(|animation| {
            animation.on_frame(dt);
            !animation.is_finished()
        });

       
        for player in &mut self.players {
            player.on_frame(dt as f64);
        }

        // 
        self.think_bots(dt as f64);

        // 
        for system in &mut self.pickup_spawn_systems {
            system.on_frame(dt as f64);

            if self.pickups.len() >= self.max_pickups {
                continue;
            }

            if let Some(mut pickup) = system.get_pickup_to_spawn() {
                let empty_positions = self
                    .walls
                    .iter()
                    .enumerate()
                    .flat_map(|(y, row)| {
                        row.iter().enumerate().filter_map(move |(x, wall)| {
                            if wall.variant() == WallType::Empty {
                                Some([x as i32, y as i32])
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<[i32; 2]>>();

                if !empty_positions.is_empty() {
                    let spawn_position =
                        empty_positions[rand::random::<usize>() % empty_positions.len()];
                    pickup.set_position(spawn_position);
                    self.pickups.push(pickup);
                    system.reset_spawn_timer();
                }
            }
        }

        // 
        if self.accumulated_time - self.last_update < self.update_interval {
            return;
        }
        self.last_update = self.accumulated_time;

        
        for i in 0..self.players.len() {
            if !self.players[i].get_is_alive() {
                continue;
            }

            
            let position = self.players[i].get_position();
            self.players[i].set_position(position);

            if let Some(direction) = self.players[i].get_pressed_direction() {
                let position = self.players[i].get_position();
                let new_position = direction.position_from(&position);
                let [x, y] = new_position;

                let is_intersecting = !is_in_bounds(x, y, self.column_count, self.row_count)
                    || self.walls[y as usize][x as usize].is_solid()
                    || self.players[..]
                        .iter()
                        .enumerate()
                        .filter(|(index, p)| *index != i && p.get_is_alive())
                        .any(|(_, p)| p.get_position() == new_position);

                if !is_intersecting {
                    self.players[i].set_position([x, y]);
                }

                self.players[i].set_direction(direction);
            }

            if self.players[i].get_is_fire_pressed() && self.players[i].shoot() {
                let position = self.players[i].get_position();
                let direction = self.players[i].get_direction();
                let bullet = Projectile::new(self.players[i].get_id(), position, *direction);
                self.bullets.push(bullet);
            }
        }

        // respawns
        self.players
            .iter_mut()
            .filter(|player| {
                !player.get_is_alive() && player.get_is_fire_pressed() && player.can_respawn()
            })
            .for_each(|player| {
                player.respawn();
                self.animations
                    .push(Animation::new_spawn(player.get_position()));
            });

       
        self.pickups.retain(|pickup| {
            let position = pickup.get_position();
            let mut players_to_pickup = self
                .players
                .iter_mut()
                .filter(|player| player.get_is_alive() && player.get_position() == *position);

            let is_picked_up = players_to_pickup.any(|player| match pickup.get_variant() {
                PickupType::Armor => player.add_armor(),
                PickupType::Health => player.add_health(),
            });

            !is_picked_up
        });

        self.update_bullets();

        use crate::transform::tile_to_world;
        use crate::constants::{CAMERA_EYE_HEIGHT, CAMERA_EYE_Z}; 
       if let Some(p0) = self.players.get(0) {
            use glam::Vec3;
            use crate::camera::GameCamera;
            use crate::transform::tile_to_world;

           
            let t = ((self.accumulated_time - self.last_update) / self.update_interval)
                .clamp(0.0, 1.0) as f32;

           
            let prev_w = tile_to_world(*p0.get_previous_position());
            let curr_w = tile_to_world(p0.get_position());
            let center = prev_w.lerp(curr_w, t);  

            let fwd_ld  = *p0.get_direction();
            let forward = GameCamera::forward_from(fwd_ld);

            
            let next_tile = fwd_ld.position_from(&p0.get_position());
            let ahead_solid = !is_in_bounds(next_tile[0], next_tile[1], self.column_count, self.row_count)
                || self.walls[next_tile[1] as usize][next_tile[0] as usize].is_solid();
            let nose = if ahead_solid { 0.0 } else { NOSE_OFFSET };

            
            let eye_target    = center + Vec3::new(0.0, EYE_HEIGHT_FP, 0.0) + forward * nose;
            let look_target   = eye_target + forward * LOOK_DIST;

            // sm
            self.camera.approach(eye_target, look_target, CAM_SMOOTH);
            self.camera.up   = Vec3::Y;
            self.camera.fovy = 60.0; // 
        }

        
        for i in 0..self.players.len() {
            if !self.players[i].get_is_alive() {
                continue;
            }

          
            let position = self.players[i].get_position();
            self.players[i].set_position(position);

            if let Some(direction) = self.players[i].get_pressed_direction() {
                let position = self.players[i].get_position();
                let new_position = direction.position_from(&position);
                let [x, y] = new_position;

                let is_intersecting = !is_in_bounds(x, y, self.column_count, self.row_count)
                    || self.walls[y as usize][x as usize].is_solid()
                    || self.players[..]
                        .iter()
                        .enumerate()
                        .filter(|(index, p)| *index != i && p.get_is_alive())
                        .any(|(_, p)| p.get_position() == new_position);

                if !is_intersecting {
                    self.players[i].set_position([x, y]);
                }

                self.players[i].set_direction(direction);
            }

            
            if self.players[i].get_is_fire_pressed() && self.players[i].shoot() {
                let position  = self.players[i].get_position();
                let direction = self.players[i].get_direction();
                let bullet    = Projectile::new(self.players[i].get_id(), position, *direction);
                self.bullets.push(bullet);
            }
        }




    }

    fn update_bullets(&mut self) {
        let bullets_length = self.bullets.len();
        let mut bullets_to_keep = vec![true; bullets_length];

        for i in 0..bullets_length {
            let bullet = &self.bullets[i];
            let position = bullet.get_position();
            let new_position = bullet.get_direction().position_from(&position);
            let [x, y] = new_position;

            // bounds
            if !is_in_bounds(x, y, self.column_count, self.row_count) {
                bullets_to_keep[i] = false;
                continue;
            }

            // walls
            let wall = &mut self.walls[y as usize][x as usize];
            if wall.is_solid() {
                wall.damage();
                self.animations.push(Animation::new_explosion([x, y]));
                bullets_to_keep[i] = false;
                continue;
            }

            // players
            let players_to_damage = self
                .players
                .iter_mut()
                .filter(|player| player.get_is_alive() && (player.get_position() == [x, y]));

            let mut is_player_hit = false;
            let is_player_killed = players_to_damage
                .map(|player| {
                    is_player_hit = true;
                    player.damage()
                })
                .take(1)
                .any(|is_killed| is_killed);

            if is_player_killed {
                if let Some(killer) = self
                    .players
                    .iter_mut()
                    .find(|p| p.get_id() == bullet.get_owner_id())
                {
                    killer.inc_kill_count();
                }
            }

            if is_player_hit {
                bullets_to_keep[i] = false;
                self.animations.push(Animation::new_explosion([x, y]));
                continue;
            }

            // bullet vs bullet
            self.bullets[i..]
                .iter()
                .enumerate()
                .skip(1)
                .filter(|(_, bullet)| {
                    let position_b = bullet.get_position();
                    new_position == *position_b || *position == *position_b
                })
                .for_each(|(right_index, _)| {
                    let right_index = right_index + i;
                    bullets_to_keep[i] = false;
                    bullets_to_keep[right_index] = false;
                    self.animations.push(Animation::new_explosion(new_position));
                });
        }

        // 
        let mut keep_iter = bullets_to_keep.iter();
        self.bullets.retain(|_| *keep_iter.next().unwrap());

        // 
        self.bullets.iter_mut().for_each(|bullet| {
            bullet.set_position(bullet.get_direction().position_from(&bullet.get_position()))
        });
    }

    pub fn reset(&mut self) {
        self.walls = generate_walls(self.column_count, self.row_count);
        self.pickups.clear();
        self.bullets.clear();
        self.animations.clear();
        self.players.iter_mut().for_each(|player| player.reset());
        self.pickup_spawn_systems
            .iter_mut()
            .for_each(|system| system.reset_spawn_timer());
        self.accumulated_time = 0.0;
        self.last_update = 0.0;
    }

    fn is_game_over(&self) -> bool {
        let alive_players_count = self.players.iter().filter(|p| p.get_is_alive()).count();
        let players_can_respawn = self.players.iter().filter(|p| p.can_respawn()).count();
        alive_players_count <= 1 && players_can_respawn <= 1
    }

    // -------- accessors for renderer --------

    pub fn animations(&self) -> &[Animation] {
        &self.animations
    }

    pub fn animations_mut(&mut self) -> &mut Vec<Animation> {
        &mut self.animations
    }

    pub fn camera(&self) -> &GameCamera {
        &self.camera
    }

    pub fn players(&self) -> &[crate::player::Player] {
        &self.players
    }

    pub fn walls(&self) -> &Vec<Vec<crate::wall::Wall>> {
        &self.walls
    }


  

    pub fn pickups(&self) -> &[crate::pickup::Pickup] {
    &self.pickups
    }

    fn dir_index(dir: crate::transform::LookDirection) -> usize {
    use crate::transform::LookDirection::*;
    match dir { Up => 0, Right => 1, Down => 2, Left => 3 }
}

fn clear_line_of_sight(&self, from: [i32; 2], to: [i32; 2]) -> bool {
    use std::cmp::{min, max};
    if from[0] == to[0] {
        let x = from[0];
        let (y0, y1) = (min(from[1], to[1]), max(from[1], to[1]));
        for y in (y0 + 1)..y1 {
            if self.walls[y as usize][x as usize].is_solid() { return false; }
        }
        true
    } else if from[1] == to[1] {
        let y = from[1];
        let (x0, x1) = (min(from[0], to[0]), max(from[0], to[0]));
        for x in (x0 + 1)..x1 {
            if self.walls[y as usize][x as usize].is_solid() { return false; }
        }
        true
    } else {
        false 
    }
}
    
    //
    fn think_bots(&mut self, dt: f64) {

        if self.state != GameState::Playing { return; }

    use crate::transform::LookDirection::{Up, Right, Down, Left};

    let human_pos = self.players[0].get_position();

    for i in 1..self.players.len() {
        if self.bot_brains[i].is_none() || !self.players[i].get_is_alive() {
            continue;
        }

        // 1) cooldowns
        {
            let br = self.bot_brains[i].as_mut().unwrap();
            br.think_timer -= dt;
            br.fire_cd     -= dt;
        }

        // snapshots
        let (think_timer, fire_cd) = {
            let br = self.bot_brains[i].as_ref().unwrap();
            (br.think_timer, br.fire_cd)
        };
        let my_pos = self.players[i].get_position();

        // 
        let mut chosen_dir = None;
        if think_timer <= 0.0 {
            let dx = human_pos[0] - my_pos[0];
            let dy = human_pos[1] - my_pos[1];

            let pref = if dx.abs() > dy.abs() {
                if dx > 0 { [Right, Down, Up, Left] } else { [Left, Down, Up, Right] }
            } else {
                if dy > 0 { [Down, Right, Left, Up] } else { [Up, Right, Left, Down] }
            };

            for dir in pref {
                let np = dir.position_from(&my_pos);
                let [x, y] = np;

                let blocked =
                    !is_in_bounds(x, y, self.column_count, self.row_count) ||
                    self.walls[y as usize][x as usize].is_solid() ||
                    self.players.iter().enumerate()
                        .any(|(j, p)| j != i && p.get_is_alive() && p.get_position() == np);

                if !blocked {
                    chosen_dir = Some(dir);
                    break;
                }
            }
        }

        // 
        let (aligned, aim_dir) = if my_pos[0] == human_pos[0] {
            (true, if human_pos[1] > my_pos[1] { Down } else { Up })
        } else if my_pos[1] == human_pos[1] {
            (true, if human_pos[0] > my_pos[0] { Right } else { Left })
        } else {
            (false, *self.players[i].get_direction())
        };

        let manhattan = (human_pos[0] - my_pos[0]).abs() + (human_pos[1] - my_pos[1]).abs();
        let los_ok = aligned && Self::los_on_walls(&self.walls, my_pos, human_pos);
        let close_override = aligned && manhattan <= 2; // shoot if very close anyway

        let want_fire = (los_ok || close_override) && fire_cd <= 0.0;

        // 
        {
            let p = &mut self.players[i];
            for k in 0..4 { p.set_move_pressed(k, false); }

            if aligned { p.set_direction(aim_dir); } // face target if same row/col

            if let Some(dir) = chosen_dir {
                p.set_move_pressed(Self::dir_index(dir), true);
            }

            p.set_fire_pressed(want_fire);
        }
        {
            let br = self.bot_brains[i].as_mut().unwrap();
            if think_timer <= 0.0 { br.think_timer = 0.18; }
            if want_fire        { br.fire_cd     = 0.60; } // tweak
        }
    }
}



    //

    




    fn los_on_walls(walls: &[Vec<crate::wall::Wall>], from: [i32; 2], to: [i32; 2]) -> bool {
        use std::cmp::{min, max};
        if from[0] == to[0] {
            let x = from[0];
            let (y0, y1) = (min(from[1], to[1]), max(from[1], to[1]));
            for y in (y0 + 1)..y1 {
                if walls[y as usize][x as usize].is_solid() { return false; }
            }
            true
        } else if from[1] == to[1] {
            let y = from[1];
            let (x0, x1) = (min(from[0], to[0]), max(from[0], to[0]));
            for x in (x0 + 1)..x1 {
                if walls[y as usize][x as usize].is_solid() { return false; }
            }
            true
        } else {
            
            false
        }
    }

    
    pub fn bullets(&self) -> &[Projectile] {
        &self.bullets
    }


    
//
    fn check_victory(&mut self) {
        use crate::game::GameState::*;
        if self.state != Playing { return; }

        let human_alive_or_can_respawn = self.players.get(0)
            .map(|p| p.get_is_alive() || p.can_respawn())
            .unwrap_or(false);

      
        let any_enemy_alive = self.players.iter().skip(1).any(|p| p.get_is_alive());

        if human_alive_or_can_respawn && !any_enemy_alive {
            self.state = Won;
            self.state_time = 0.0;
        } else if !human_alive_or_can_respawn {
            self.state = Lost;
            self.state_time = 0.0;
        }
    }


//


     pub fn state(&self) -> GameState { self.state }
    pub fn state_time(&self) -> f32 { self.state_time }


     pub fn resume(&mut self) {
        self.state = GameState::Playing;
        self.state_time = 0.0;
    }



}






#[derive(Clone, Copy, Debug)]
struct BotBrain {
    think_timer: f64,  //
    fire_cd: f64,      
}

impl Default for BotBrain {
    fn default() -> Self { Self { think_timer: 0.0, fire_cd: 0.0 } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Menu,  
    Playing,
    Won,
    Lost,

}


