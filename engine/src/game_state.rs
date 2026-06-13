use std::{collections::{HashMap, VecDeque}, rc::Rc};

use render::EntityRenderer;

use crate::{CustomEntity, camera::Camera, transform::Transform};

pub struct InternalEntity {
    pub id: u64,
    pub drawer: EntityRenderer,
    pub transform: Transform,
    pub is_dead: bool
}

struct Entity {
    internal: InternalEntity,
    custom: Box<dyn CustomEntity>
}

pub struct GameState {
    pub(crate) entities: Vec<Entity>,
    pub(crate) camera: Camera,
    uid_incrementer: u64
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            camera: Camera::default(),
            uid_incrementer: 0
        }
    }
}

        //self.entities.insert(id, entity_holder);
impl GameState {
    pub fn new_entity<T: CustomEntity>(&mut self, custom: T) -> u64 {
        let uid = self.uid_incrementer;
        self.uid_incrementer += 1;

        let internal = InternalEntity { id: uid, drawer: EntityRenderer::init(custom.mesh()), transform: Transform::default(), is_dead: false };
        let mut entity = Entity { internal, custom: Box::new(custom) };
        entity.custom.start(self);

        self.entities.push(entity);

        uid
    }

    pub fn update(&mut self, delta_time:f32) {
        let entities_len = self.entities.len();
        let mut new_vec: Vec<Entity> = Vec::with_capacity(entities_len);
        let mut i = entities_len-1;
        while i >= 0 {
            let a = self.entities.pop();
            if a.is_none() {
                i -= 1;
                continue;
            }
            let mut a = a.unwrap();
            a.custom.update(self, delta_time);
            if !a.internal.is_dead {
                new_vec.push(a);
            }
            i -= 1;
        }
        self.entities = new_vec;
    }
}