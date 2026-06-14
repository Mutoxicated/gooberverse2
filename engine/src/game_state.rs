use std::{sync::mpsc::{Receiver, Sender}, thread, time::Duration};

use render::{EntityRenderer, RenderObject};

use crate::{CustomEntity, app::ToApp::{self, CreateEntityRenderer, StartRender}, camera::Camera, transform::Transform};

pub struct InternalEntity {
    pub(crate) id: u64,
    pub(crate) drawer: Option<EntityRenderer>,
    pub transform: Transform,
    pub(crate) is_dead: bool,
}

impl InternalEntity {
    pub fn kill(&mut self) {
        self.is_dead = true;
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

pub(crate) struct Entity {
    internal: InternalEntity,
    custom: Box<dyn CustomEntity>,
}

impl Entity {
    pub fn get_render_object(&self) -> RenderObject {
        RenderObject { 
            model_matrix: self.internal.transform.model_matrix(), 
            drawer: self.internal.drawer.clone().unwrap(), 
            shaders_to_use: self.custom.shaders_to_use() 
        }
    }
}

pub enum ToGameState {
    GetEntityRenderer(EntityRenderer)
}

pub struct GameState {
    pub(crate) entities: Vec<Entity>,
    camera: Camera,
    uid_incrementer: u64,
    /// fixed timestep in seconds
    fixed_timestep: u64,
    to_app: Sender<ToApp>,
    inbox: Receiver<ToGameState>
}

//self.entities.insert(id, entity_holder);
impl GameState {
    pub(crate) fn new(fixed_timestep: u64, to_app: Sender<ToApp>, inbox: Receiver<ToGameState>) -> Self {
        Self {
            entities: Vec::new(),
            camera: Camera::default(),
            uid_incrementer: 0,
            fixed_timestep,
            to_app,
            inbox
        }
    }

    pub(crate) fn start_runtime(mut self) {
        thread::spawn(move || {// game thread
            loop {
                thread::sleep(Duration::from_millis(self.fixed_timestep));
                let robjs = self.fixed_update();
                let res = self.to_app.send(StartRender(robjs, self.camera.clone()));
                if res.is_err() {
                    break;
                }
            }
        });
    } 

    pub fn new_entity<T: CustomEntity>(&mut self, custom: T) -> u64 {
        let uid = self.uid_incrementer;
        self.uid_incrementer += 1;

        let internal = InternalEntity {
            id: uid,
            drawer: None,
            transform: Transform::default(),
            is_dead: false,
        };
        let mut entity = Entity {
            internal,
            custom: Box::new(custom),
        };
        entity.custom.start(&mut entity.internal, self);

        self.entities.push(entity);

        uid
    }

    pub fn fixed_update(&mut self) -> Vec<RenderObject> {
        let entities_len = self.entities.len();
        
        let mut robjs = Vec::<RenderObject>::with_capacity(entities_len);
        let mut i = entities_len as isize - 1;
        while i >= 0 {
            let a = self.entities.pop();
            if a.is_none() {
                i -= 1;
                continue;
            }
            let mut a = a.unwrap();
            a.custom.fixed_update(&mut a.internal, self, self.fixed_timestep as f32);
            if a.internal.is_dead {
                i -= 1;
                continue;
            }
            if a.internal.drawer.is_none() && !a.custom.mesh().is_invalid() {
                let _ = self.to_app.send(CreateEntityRenderer(a.custom.mesh().clone()));
                let res = self.inbox.recv().unwrap();
                a.internal.drawer = match res {
                    ToGameState::GetEntityRenderer(m) => Some(m)
                }
            }
            if a.internal.drawer.is_some() {
                robjs.push(a.get_render_object());
            }
            self.entities.push(a);
            i -= 1;
        }
        robjs
    }
}
