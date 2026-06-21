use std::{any::TypeId, collections::HashMap, sync::Arc};

use gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT};
use glam::{Mat4, Vec3};
use render::{EntityRenderer, ObjectShader, RenderObject, mesh::Mesh};

use crate::{WORLD_SCALE, get_gl_error};

#[derive(Clone)]
pub struct CameraRenderInfo {
    pub proj_mat: Mat4,
    pub view_mat: Mat4,
    pub pos: Vec3,
}

#[derive(Clone)]
pub struct Batch {
    pub objs: Arc<[RenderObject]>,
    pub camera: CameraRenderInfo,
}

impl Batch {
    pub fn new(objs: Arc<[RenderObject]>, camera_info: CameraRenderInfo) -> Self {
        Self {
            objs,
            camera: camera_info,
        }
    }
}

pub struct Renderer {
    pub shaders: Vec<ObjectShader>,
    entity_renderers: HashMap<TypeId, EntityRenderer>,
    _fixed_timestep: f32,
}

impl Renderer {
    pub fn new(shaders: Vec<ObjectShader>, game_fixed_timestep: f32) -> Self {
        Self {
            shaders,
            entity_renderers: HashMap::new(),
            _fixed_timestep: game_fixed_timestep,
        }
    }

    pub fn render(&mut self, batch: &Batch) {
        use render::uniform::*;
        unsafe {
            gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
        }
        let camera = &batch.camera;
        let objs = &batch.objs;

        let (proj, view) = (camera.proj_mat, camera.view_mat);
        for os in &self.shaders {
            os.shader.activate();
            os.shader.set_mat4(PROJ, proj);
            os.shader.set_mat4(VIEW, view);
            os.shader.set_float(SCALE, WORLD_SCALE!());
        }
        let special_unis = render::SpecialUnis {
            cam_pos: camera.pos,
        };
        for obj in objs.iter() {
            if !self.entity_renderers.contains_key(&obj.entity_type_id) {
                continue;
            }
            for shader_index in &obj.shaders_to_use {
                let os = &self.shaders[*shader_index as usize];
                os.shader.activate();
                os.shader.set_mat4(MODEL, obj.model_matrix);
                os.info.set_special_uniforms(&special_unis, &os.shader);
                self.entity_renderers[&obj.entity_type_id].draw();
            }
        }
        get_gl_error!();
    }

    pub fn new_entity_renderer(&mut self, mesh: &Mesh, tid: TypeId) {
        self.entity_renderers
            .insert(tid, EntityRenderer::init(mesh));
    }

    pub fn remove_entity_renderer(&mut self, tid: TypeId) {
        self.entity_renderers.remove(&tid);
    }
}
