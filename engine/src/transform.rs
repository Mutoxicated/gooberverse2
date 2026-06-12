use glam::{Mat4, Quat, Vec3};

use crate::WORLD_SCALE;

pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE
        }
    }
}

impl Transform {
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position*WORLD_SCALE!())
    }

    pub fn set_rotation(&mut self, rot:Vec3) {
        self.rotation = Quat::from_euler(glam::EulerRot::ZXY, rot.z, rot.x, rot.y);
    }

    pub fn set_position(&mut self, pos:Vec3) {
        self.position = pos;
    }

    pub fn set_scale(&mut self, scale:Vec3) {
        self.scale = scale;
    }
}