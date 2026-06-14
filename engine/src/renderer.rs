use std::sync::mpsc::Sender;

use render::{ObjectShader, RenderObject};

use crate::{WORLD_SCALE, app::ToApp, camera::Camera};

pub struct Batch {
    pub objs: Vec<RenderObject>,
    pub camera: Camera,
}

impl Batch {
    pub fn new(objs: Vec<RenderObject>, camera_info: Camera) -> Self {
        Self {
            objs,
            camera: camera_info,
        }
    }
}

pub struct Renderer {
    pub to_core: Sender<ToApp>,
    pub shaders: Vec<ObjectShader>,
}

impl Renderer {
    pub fn new(
        shaders: Vec<ObjectShader>,
        to_core: Sender<ToApp>,
    ) -> Self {
        Self {
            shaders,
            to_core,
        }
    }
}
