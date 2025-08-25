use glam::Vec3;
use crate::transform::LookDirection;

#[derive(Clone, Copy, Debug)]
pub struct GameCamera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fovy: f32,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            eye: Vec3::new(0.0, 7.0, 6.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fovy: 60.0,
        }
    }
}

impl GameCamera {
    /// Grid facing → world forward (X/Z plane; Y up)
    pub fn forward_from(d: LookDirection) -> Vec3 {
        match d {
            LookDirection::Up    => Vec3::new( 0.0, 0.0, -1.0),
            LookDirection::Down  => Vec3::new( 0.0, 0.0,  1.0),
            LookDirection::Left  => Vec3::new(-1.0, 0.0,  0.0),
            LookDirection::Right => Vec3::new( 1.0, 0.0,  0.0),
        }
    }

    /// First‑person camera. `nose` > 0 pushes forward, < 0 pulls back.
    pub fn set_first_person(&mut self, world_pos: Vec3, forward: Vec3, eye_height: f32, nose: f32) {
        self.eye    = world_pos + Vec3::new(0.0, eye_height, 0.0) + forward * nose;
        self.target = self.eye + forward * 10.0;
        self.up     = Vec3::Y;
        self.fovy   = 70.0;
    }

    /// Optional: smooth move the camera toward targets
    pub fn approach(&mut self, eye_target: Vec3, look_target: Vec3, alpha: f32) {
        self.eye    = self.eye.lerp(eye_target,  alpha);
        self.target = self.target.lerp(look_target, alpha);
    }


    /// Optional: compute desired eye/target for FPS
    pub fn first_person_targets(world_pos: Vec3, forward: Vec3, eye_height: f32, nose: f32) -> (Vec3, Vec3) {
        let eye = world_pos + Vec3::new(0.0, eye_height, 0.0) + forward * nose;
        (eye, eye + forward * 10.0)
    }
}
