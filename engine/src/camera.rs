use glam::{Mat4, Vec3};

use crate::{WORLD_SCALE, renderer::CameraRenderInfo};

#[derive(Clone)]
pub struct Camera {
    pub front: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vec3,
    pub fov: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub screen_size: (i32, i32),
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            front: Vec3::new(0.0, 0.0, 1.0),
            pitch: 0.0,
            yaw: 90.0,
            position: Vec3::ZERO,
            fov: 90_f32,
            z_near: 0.3,
            z_far: 300.0,
            screen_size: (0, 0),
        }
    }
}

impl Camera {
    pub fn rotate_from_raw_cursor_data(&mut self, mut data: glam::Vec2, fixed_dt: f32) {
        data.x *= 8.0 * fixed_dt;
        data.y *= 8.0 * fixed_dt;

        self.yaw -= data.x;
        self.pitch = (self.pitch - data.y).clamp(-89.0, 89.0);
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        self.front = Vec3 {
            x: yaw_rad.cos() * pitch_rad.cos(),
            y: pitch_rad.sin(),
            z: yaw_rad.sin() * pitch_rad.cos(),
        }
        .normalize();
    }

    pub fn view_matrix(&self) -> Mat4 {
        let world_pos = self.position * WORLD_SCALE!();
        Mat4::look_at_lh(world_pos, world_pos + self.front, crate::UP)
    }

    pub fn proj_matrix(&self) -> Mat4 {
        let screen_ratio = self.screen_size.0 as f32 / self.screen_size.1 as f32;
        Mat4::perspective_lh(self.fov.to_radians(), screen_ratio, self.z_near, self.z_far)
    }

    pub fn render_info(&self) -> CameraRenderInfo {
        CameraRenderInfo {
            proj_mat: self.proj_matrix(),
            view_mat: self.view_matrix(),
            pos: self.position,
        }
    }
}
