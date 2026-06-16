
use std::{collections::HashMap, sync::Arc};

use gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT};
use glam::{Mat4, Vec3};
use render::{EntityRenderer, Mesh, ObjectShader, RenderObject};

use crate::{WORLD_SCALE, get_gl_error};

pub(crate) struct CameraRenderInfo {
    pub(crate) proj_mat: Mat4,
    pub(crate) view_mat: Mat4,
    pub(crate) pos: Vec3
}

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
    entity_renderers: HashMap<u64, EntityRenderer>,
}

impl Renderer {
    pub fn new(shaders: Vec<ObjectShader>) -> Self {
        Self {
            shaders,
            entity_renderers: HashMap::new(),
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
            if !self.entity_renderers.contains_key(&obj.entity_id) {
                continue;
            }
            for shader_index in &obj.shaders_to_use {
                let os = &self.shaders[*shader_index as usize];
                os.shader.activate();
                os.shader.set_mat4(MODEL, obj.model_matrix);
                os.info.set_special_uniforms(&special_unis, &os.shader);
                self.entity_renderers[&obj.entity_id].draw();
            }
        }
        get_gl_error!();
    }

    pub fn new_entity_renderer(&mut self, mesh: &'static Mesh, eid: u64) {
        self.entity_renderers
            .insert(eid, EntityRenderer::init(mesh));
    }

    pub fn remove_entity_renderer(&mut self, eid: u64) {
        self.entity_renderers.remove(&eid);
    }
}
