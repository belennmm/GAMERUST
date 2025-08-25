use raylib::prelude::*;

mod animation;
mod constants;
mod camera;
mod game;
mod pickup;
mod player;
mod projectile;
mod render;
mod transform;
mod wall;

use crate::game::{Game, GameState};
use crate::render::{draw_scene_3d, SceneTextures};
use raylib::prelude::{KeyboardKey, Vector2};

fn main() {
    // --- Raylib window/context ---
    let (mut rl, thread) = raylib::init()
        .size(1024, 768)
        .title("Battle Kill")
        .vsync()
        .build();

    rl.set_target_fps(60);

    // --- Load atlas ---
    let atlas = rl
        .load_texture(&thread, "resources/tanks.png")
        .expect("failed to load resources/tanks.png");
    let tex = SceneTextures::new(atlas);

    // --- Create game ---
    let mut game = Game::new_default();

    // --- Build Camera3D from game state ---
    let gc0 = game.camera();
    let mut cam = Camera3D::perspective(
        Vector3::new(gc0.eye.x, gc0.eye.y, gc0.eye.z),
        Vector3::new(gc0.target.x, gc0.target.y, gc0.target.z),
        Vector3::new(gc0.up.x, gc0.up.y, gc0.up.z),
        60.0,
    );

    // --- Main loop ---
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // --------- INPUT (must be BEFORE begin_drawing) ---------
        // Global hotkeys
        let pressed_r      = rl.is_key_pressed(KeyboardKey::KEY_R);
        let pressed_enter  = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
         let pressed_e = rl.is_key_pressed(KeyboardKey::KEY_E);

        if pressed_r {
            // full fresh start -> back to menu (Game::reset sets state = Menu)
            game.reset();
        }

        if pressed_e {
            break; // <- exit the loop -> closes program
        }


        // feed movement/fire input for human
        game.poll_input_raylib(&rl);

        // if we are in the menu and user hits Enter, start the game
        if game.state() == GameState::Menu && pressed_enter {
            game.resume(); // sets state = Playing, resets state_time
        }

        // --------- UPDATE ---------
        game.update_dt(dt);

        // sync camera from game camera
        let gcam = game.camera();
        cam.position = Vector3::new(gcam.eye.x, gcam.eye.y, gcam.eye.z);
        cam.target   = Vector3::new(gcam.target.x, gcam.target.y, gcam.target.z);
        cam.up       = Vector3::new(gcam.up.x, gcam.up.y, gcam.up.z);
        cam.fovy     = gcam.fovy;

        // --------- DRAW ---------
        let mut d2 = rl.begin_drawing(&thread);

        match game.state() {
            GameState::Menu => {
                d2.clear_background(Color::BLACK);
                render::draw_menu(&mut d2);
            }

            _ => {
                d2.clear_background(Color::BLACK);

                // 3D world
                {
                    let mut d3 = d2.begin_mode3D(cam);
                    draw_scene_3d(&mut d3, &cam, &game, &tex, Some(0)); // skip p0 billboard in FPS
                }

                // 2D overlays
                render::draw_minimap_2d(&mut d2, &game, Vector2::new(10.0, 40.0), 8);
                render::draw_hud_bar(&mut d2, &game);
                render::draw_state_banner_2d(&mut d2, &game);
                d2.draw_fps(10, 10);
            }
        }
    }
}
