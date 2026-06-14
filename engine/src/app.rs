use std::sync::mpsc::{Receiver, Sender};

use gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use render::{EntityRenderer, Mesh, RenderObject};

use crate::{WORLD_SCALE, camera::Camera, game_state::{GameState, ToGameState::{self, GetEntityRenderer}}, get_gl_error, renderer::{Batch, Renderer}};

pub enum ToApp {
    StartRender(Vec<RenderObject>, Camera),
    CreateEntityRenderer(Mesh)
}

// NOTE: The order of some fields is VERY important: glfw NEEDS to be 
// the last field, because it has to always get dropped last.
pub struct App {
    pub inbox: Receiver<ToApp>,
    pub to_game_state: Sender<ToGameState>,
    pub renderer: Renderer,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
    pub glfw: Glfw,
}

impl App {
    pub fn start_runtime(mut self, mut func: impl FnMut(&mut App)) {
        let mut delta_time: f32;
        let mut prev_time = self.glfw.get_time();
        while !self.window.should_close() {// app thread
            let time = self.glfw.get_time();
            delta_time = (time - prev_time) as f32;
            prev_time = time;

            func(&mut self);

            self.glfw.poll_events();
            self.check_inbox();
        }
    }
    pub fn check_inbox(&mut self) {
        use ToApp::*;

        let result = self.inbox.try_recv();
        if result.is_err() {
            return;
        }
        match result.unwrap() {
            StartRender(objs, cam) => {
                println!("[App] StartRender");
                self.render(Batch { objs, camera: cam });
            }
            CreateEntityRenderer(mesh) => {
                println!("[App] CreateEntityRenderer");
                let _ = self.to_game_state.send(GetEntityRenderer(EntityRenderer::init(mesh).unwrap()));
            }
        }
    }

    fn render(&mut self, batch: Batch) {
        use render::uniform::*;
        unsafe {
            gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
        }
        let camera = batch.camera;
        let objs = batch.objs;

        let (proj, view) = (camera.proj_matrix(), camera.view_matrix());
        for os in &self.renderer.shaders {
            os.shader.activate();
            os.shader.set_mat4(PROJ, proj);
            os.shader.set_mat4(VIEW, view);
            os.shader.set_float(SCALE, WORLD_SCALE!());
        }
        let special_unis = render::SpecialUnis {
            cam_pos: camera.position,
        };
        for obj in objs {
            for shader_index in obj.shaders_to_use {
                let os = &self.renderer.shaders[shader_index as usize];
                os.shader.activate();
                os.shader.set_mat4(MODEL, obj.model_matrix);
                os.info.set_special_uniforms(&special_unis, &os.shader);
                obj.drawer.draw();
            }
        }
        get_gl_error!();
        self.window.swap_buffers();
    }

    pub fn set_bg_color(&self, r:f32, g:f32, b:f32) {
        unsafe {
            gl::ClearColor(r, g, b, 1.0);
        }
    }
}
