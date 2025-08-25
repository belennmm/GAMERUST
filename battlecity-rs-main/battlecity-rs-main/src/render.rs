use raylib::prelude::*;
use crate::animation::Animation;
use crate::game::Game;
use crate::transform::tile_to_world;
use crate::constants::{SPRITE_HEIGHT, WALL_HEIGHT}; 
use crate::wall::WallType;
use raylib::prelude::Color;
use crate::pickup::PickupType;


pub trait GameRenderObject {
    fn is_visible(&self) -> bool;
    fn get_frame(&self) -> &[f64; 4];   
    fn get_position(&self) -> &[i32; 2];
    fn get_previous_position(&self) -> &[i32; 2];
}

pub struct SceneTextures {
    pub atlas: Texture2D,
    pub atlas_size: (f32, f32),
}

impl SceneTextures {
    pub fn new(atlas: Texture2D) -> Self {
        // read dims BEFORE moving atlas into the struct
        let w = atlas.width() as f32;
        let h = atlas.height() as f32;
        Self { atlas, atlas_size: (w, h) }
    }
}


pub fn draw_scene_3d(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    camera: &Camera3D,
    game: &Game,
    tex: &SceneTextures,
    skip_player_id: Option<u32>,
) {
   
    draw_walls_3d(d, camera, game, tex);

  
    draw_pickups_3d(d, camera, game, tex);

    //  players
    for p in game.players() {
        if let Some(skip) = skip_player_id {
            if p.get_id() == skip { continue; }
        }
        draw_player_billboard(d, camera, p, tex);
    }

    //
    for anim in game.animations() {
        draw_animation_billboard(d, camera, anim, tex);
    }

    // bullets
     // bullets  
    draw_bullets_3d(d, camera, game);

    // pickups/animations
    for anim in game.animations() { draw_animation_billboard(d, camera, anim, tex); }
    draw_pickups_3d(d, camera, game, tex);


}


fn draw_animation_billboard(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    camera: &Camera3D,
    anim: &Animation,
    tex: &SceneTextures,
) {
    let [sx, sy, sw, sh] = *anim.get_frame();
    let source = Rectangle { x: sx as f32, y: sy as f32, width: sw as f32, height: sh as f32 };

    let pos = Vector3::new(anim.position.x, anim.position.y + anim.size * 0.5, anim.position.z);
    let size = Vector2::new(anim.size, anim.size); 

    d.draw_billboard_rec(
        *camera,
        &tex.atlas,
        source,
        pos,
        size,                    
        Color::WHITE,
    );
}

fn draw_player_billboard(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    camera: &Camera3D,
    player: &crate::player::Player,
    tex: &SceneTextures,
) {
    if !player.is_visible() { return; }

    let [sx, sy, sw, sh] = *player.frame_uv();
    let source = Rectangle { x: sx as f32, y: sy as f32, width: sw as f32, height: sh as f32 };

    let tile = player.get_position();
    let wp = tile_to_world(tile); // z)
    let pos = Vector3::new(wp.x, wp.y + SPRITE_HEIGHT * 0.5, wp.z);

    let aspect = (source.width / source.height).abs().max(1e-6);
    let size = Vector2::new(SPRITE_HEIGHT * aspect, SPRITE_HEIGHT);

    d.draw_billboard_rec(
        *camera,
        &tex.atlas,
        source,
        pos,
        size,                
        Color::WHITE,
    );
}

fn draw_walls_3d(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    camera: &Camera3D,
    game: &Game,
    tex: &SceneTextures,
) {

    let cube_size = Vector3::new(1.0, WALL_HEIGHT, 1.0);
    use crate::wall::wall_center_for;
    use raylib::prelude::Vector2;

   for row in game.walls() {
        for wall in row {
            if !wall.is_visible() { continue; }

            match wall.variant() {
                WallType::Border => {
                   
                    let c = wall_center_for(*wall.get_position());
                    let pos = Vector3::new(c.x, c.y, c.z);
                    d.draw_cube_v(pos, cube_size, Color::BROWN);
                    d.draw_cube_wires_v(pos, cube_size, Color::DARKBROWN);
                }
                _ => {
                   
                    let [sx, sy, sw, sh] = *wall.get_frame();
                    let source = Rectangle { x: sx as f32, y: sy as f32, width: sw as f32, height: sh as f32 };

                  
                    let c = wall_center_for(*wall.get_position());
                    let pos = Vector3::new(c.x, c.y, c.z);

                  
                    let size = Vector2::new(1.0, WALL_HEIGHT);

                    d.draw_billboard_rec(
                        *camera,
                        &tex.atlas,
                        source,
                        pos,
                        size,
                        Color::WHITE,
                    );
                }
            }
        }
    }
}

pub fn draw_minimap_2d(
    d2: &mut RaylibDrawHandle,
    game: &Game,
    origin: Vector2,
    tile_px: i32,
) {
    let walls = game.walls();
    if walls.is_empty() { return; }

    let rows = walls.len() as i32;
    let cols = walls[0].len() as i32;

    let map_w = cols * tile_px;
    let map_h = rows * tile_px;

    // background panel
    d2.draw_rectangle(
        origin.x as i32 - 4,
        origin.y as i32 - 4,
        map_w + 8,
        map_h + 8,
        Color::new(0, 0, 0, 180),
    );

    // tiles
    for y in 0..rows {
        for x in 0..cols {
            let wall = &walls[y as usize][x as usize];
            let color = match wall.variant() {
                WallType::Empty    => Color::new(0, 0, 0, 0),     
                WallType::Brick    => Color::ORANGE,
                WallType::Concrete => Color::GRAY,
                WallType::Net      => Color::GREEN,
                WallType::Border => Color::BROWN,
              
            };
            if color.a > 0 {
                d2.draw_rectangle(
                    origin.x as i32 + x * tile_px,
                    origin.y as i32 + y * tile_px,
                    tile_px,
                    tile_px,
                    color,
                );
            }
        }
    }

    
    for (idx, p) in game.players().iter().enumerate() {
        let [px, py] = p.get_position();
        let cx = origin.x as i32 + px * tile_px + tile_px / 2;
        let cy = origin.y as i32 + py * tile_px + tile_px / 2;

        let col = if idx == 0 { Color::YELLOW } else { Color::BLUE };
        d2.draw_circle(cx, cy, (tile_px as f32 * 0.35).max(2.0), col);
    }


    for p in game.pickups() {
    let [px, py] = p.get_position();
    let cx = origin.x as i32 + px * tile_px + tile_px / 2;
    let cy = origin.y as i32 + py * tile_px + tile_px / 2;

    let dot_color = match p.get_variant() {
        PickupType::Health => Color::PINK,
        PickupType::Armor  => Color::SKYBLUE,
    };

    
    d2.draw_rectangle(cx - 2, cy - 2, 4, 4, dot_color);
  
}
}

fn draw_pickups_3d(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    camera: &Camera3D,
    game: &Game,
    tex: &SceneTextures,
) {
    let t = 0.0_f32; 
    for p in game.pickups() {
        let [sx, sy, sw, sh] = *p.get_frame();
        let source = Rectangle { x: sx as f32, y: sy as f32, width: sw as f32, height: sh as f32 };

        let tile = *p.get_position();
        let wp = crate::transform::tile_to_world(tile);
        let y = wp.y + 0.45 + (t * 3.0).sin() * 0.05;   
        let pos = Vector3::new(wp.x, y, wp.z);

        let size = Vector2::new(0.6, 0.6);             
        d.draw_billboard_rec(
            *camera,
            &tex.atlas,
            source,
            pos,
            size,
            Color::WHITE,
        );
    }
}

pub fn draw_hud_bar(
    d2: &mut RaylibDrawHandle,
    game: &crate::game::Game,
) {
    use crate::constants::{PLAYER_MAX_HEALTH, PLAYER_MAX_ARMOR};

    
    let Some(p0) = game.players().get(0) else { return; };

    let screen_w = d2.get_screen_width();
    let screen_h = d2.get_screen_height();

    // --- layout ---
    let pad     = 10;
    let height  = 64;               // HUD bar height
    let y       = screen_h - height - pad;
    let x       = pad;
    let width   = screen_w - pad*2;

    // panel background
    d2.draw_rectangle_rounded(
        Rectangle { x: x as f32, y: y as f32, width: width as f32, height: height as f32 },
        0.12,
        10,
        Color::new(0, 0, 0, 180),
    );

    // inner padding
    let inner_pad = 12;
    let mut cursor_x = x + inner_pad;
    let center_y = y + height/2;

    // --- HEALTH ---
    {
        let label = "HEALTH";
        d2.draw_text(label, cursor_x, center_y - 24, 18, Color::LIME);

        let bar_x = cursor_x;
        let bar_y = center_y - 10;
        let bar_w = 220;
        let bar_h = 18;

        // bar background
        d2.draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::DARKGRAY);
        // fill
        let h_cur = p0.get_health().min(PLAYER_MAX_HEALTH) as f32;
        let h_max = PLAYER_MAX_HEALTH as f32;
        let fill_w = ((h_cur / h_max) * bar_w as f32).round() as i32;
        d2.draw_rectangle(bar_x, bar_y, fill_w, bar_h, Color::GREEN);

        // numbers
        let txt = format!("{}/{}", p0.get_health(), PLAYER_MAX_HEALTH);
        d2.draw_text(&txt, bar_x + bar_w + 10, bar_y - 2, 20, Color::WHITE);

        cursor_x += bar_w + 170; // advance layout
    }

    // --- ARMOR ---
    {
        let label = "ARMOR";
        d2.draw_text(label, cursor_x, center_y - 24, 18, Color::SKYBLUE);

        let bar_x = cursor_x;
        let bar_y = center_y - 10;
        let bar_w = 220;
        let bar_h = 18;

        d2.draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::DARKGRAY);
        let a_cur = p0.get_armor().min(PLAYER_MAX_ARMOR) as f32;
        let a_max = PLAYER_MAX_ARMOR as f32;
        let fill_w = ((a_cur / a_max) * bar_w as f32).round() as i32;
        d2.draw_rectangle(bar_x, bar_y, fill_w, bar_h, Color::BLUE);

        let txt = format!("{}/{}", p0.get_armor(), PLAYER_MAX_ARMOR);
        d2.draw_text(&txt, bar_x + bar_w + 10, bar_y - 2, 20, Color::WHITE);

        cursor_x += bar_w + 170;
    }

    // --- LIVES & KILLS (right-aligned block) ---
    {
        let right_pad = 16;
        let block_w = 200;
        let rx = screen_w - block_w - right_pad;

        // lives
        let lives_txt = format!("Lives: {}", p0.get_lives());
        d2.draw_text(&lives_txt, rx, center_y - 24, 20, Color::YELLOW);

        // kills
        let kills_txt = format!("Kills: {}", p0.get_kills());
        d2.draw_text(&kills_txt, rx, center_y + 2, 20, Color::ORANGE);
    }
}


fn draw_bullets_3d(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    camera: &Camera3D,
    game: &crate::game::Game,
) {
    for b in game.bullets() {
        let tile = *b.get_position();
        let wp = crate::transform::tile_to_world(tile);
        // small glowing billboard
        let pos = Vector3::new(wp.x, wp.y + 0.45, wp.z);
        let size = Vector2::new(0.2, 0.2);
        // simple colored quad (no texture): use a tiny cube for visibility
        d.draw_cube_v(Vector3::new(pos.x, pos.y, pos.z), Vector3::new(0.15, 0.15, 0.15), Color::YELLOW);
    }
}

use crate::game::GameState;

pub fn draw_state_banner_2d(d: &mut RaylibDrawHandle, game: &crate::game::Game) {
    let state = game.state();
    if state == GameState::Playing { return; }

    let sw = d.get_screen_width() as f32;
    let sh = d.get_screen_height() as f32;

    // Fade-in over 1s
   let alpha = game.state_time().min(1.0);
    let bg = Color::new(0, 0, 0, (160.0 * alpha) as u8);

    d.draw_rectangle(0, 0, sw as i32, sh as i32, bg);

    let (title, color) = match state {
        GameState::Won    => ("YOU WIN!", Color::YELLOW),
        GameState::Lost   => ("YOU LOSE", Color::RED),
        GameState::Playing
        | GameState::Menu => return, // nothing to draw in these states
    };

    let title_size = 40;
    let hint_size  = 20;

    let tw = d.measure_text(title, title_size) as f32;
    let hw = d.measure_text("Press R to restart", hint_size) as f32;

    let cx = sw * 0.5;
    let cy = sh * 0.45;

    d.draw_text(title, (cx - tw * 0.5) as i32, (cy - 20.0) as i32, title_size, color);
    d.draw_text(
        "Press R to restart",
        (cx - hw * 0.5) as i32,
        (cy + 30.0) as i32,
        hint_size,
        Color::RAYWHITE
    );
}

pub fn draw_menu(d: &mut RaylibDrawHandle) {
    use raylib::prelude::Color;

    let sw = d.get_screen_width();
    let sh = d.get_screen_height();

    d.clear_background(Color::BLACK);

    let title = " BATTLE KILL";
    let rules = [
        "Rules:",
        "1. Destroy all enemy tanks to win.",
        "2. Collect hearts for health.",
        "3. Collect shields for armor.",
        "4. Avoid enemy bullets!",
    ];
    let hint = "Press ENTER to Start";

    let title_size = 40;
    let text_size = 20;

    let tw = d.measure_text(title, title_size);
    d.draw_text(title, (sw/2 - tw/2), (sh/4), title_size, Color::YELLOW);

    let mut y = sh/3;
    for line in rules.iter() {
        let lw = d.measure_text(line, text_size);
        d.draw_text(line, (sw/2 - lw/2), y, text_size, Color::RAYWHITE);
        y += 30;
    }

    let hw = d.measure_text(hint, text_size);
    d.draw_text(hint, (sw/2 - hw/2), y + 40, text_size, Color::GREEN);
}
