use std::sync::mpsc::{Receiver, Sender};

use gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT};
use render::{ObjectShader, RenderObject};

use crate::{WORLD_SCALE, app::ToApp, camera::Camera, utils::get_gl_error};

pub enum ToRenderer {
    Render(Vec<RenderObject>, Camera),
}

pub struct Renderer {
    pub shaders: Vec<ObjectShader>,
    pub from_core: Receiver<ToRenderer>,
    pub to_core: Sender<ToApp>,
}

impl Renderer {
    pub fn new(shaders: Vec<ObjectShader>, from_core: Receiver<ToRenderer>, to_core: Sender<ToApp>) -> Self {
        Self {
            shaders,
            from_core,
            to_core
        }
    }
    
    fn runtime_loop(mut self) {
        use ToRenderer::*;

        while let Ok(msg) = self.from_core.recv() {
            match msg {
                Render(objs, camera) => {
                    self.render(objs, camera);
                }
            }
        }
    }

    fn render(&mut self, objs: Vec<RenderObject>, camera: Camera) {
        unsafe {
            gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
        }

        let (proj, view) = (camera.proj_matrix(), camera.view_matrix());
        for os in &self.shaders {
            os.shader.activate();
            os.shader.set_mat4("proj", proj);
            os.shader.set_mat4("view", view);
            os.shader.set_float("scale", WORLD_SCALE!());
        }
        let special_unis = render::SpecialUnis {
            cam_pos: camera.position,
        };
        for obj in objs {
            for shader_index in obj.shaders_to_use {
                let os = &self.shaders[shader_index as usize];
                os.shader.activate();
                os.shader.set_mat4("model", obj.model_matrix);
                os.info.set_special_uniforms(&special_unis, &os.shader);
                obj.drawer.draw();
            }
        }
        get_gl_error();
    }
}