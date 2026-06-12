use glam::{Mat4, Vec3};

use crate::WORLD_SCALE;

pub struct Camera {
    pub front: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vec3,
    pub fov: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub screen_size: (i32,i32)
}

impl Default for Camera {
    fn default() -> Self {
        Self { front: Vec3::new(0.0, 0.0, -1.0), pitch: 0.0, yaw: -90.0, position: Vec3::ZERO, fov: 90_f32, z_near: 0.4, z_far: 300.0, screen_size: (0,0) }
    }
}

impl Camera {
    pub const UP:Vec3 = Vec3::new(0.0, 1.0, 0.0);

    pub fn view_matrix(&self) -> Mat4 {
        let world_pos = self.position*WORLD_SCALE!();
        Mat4::look_at_rh(world_pos, world_pos+self.front, Camera::UP)
    }

    pub fn proj_matrix(&self) -> Mat4 {
        let screen_ratio = self.screen_size.0 as f32 / self.screen_size.1 as f32;
        Mat4::perspective_rh(self.fov.to_radians(), screen_ratio, self.z_near, self.z_far)
    }
}