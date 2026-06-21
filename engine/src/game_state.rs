use std::{
    any::TypeId,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    thread,
    time::Duration,
};

use crate::{
    CustomEntity, GameCallbacks,
    app::ToApp::{self, CreateEntityRenderer, InputRequest, RemoveEntityRenderer, StartRender},
    camera::Camera,
    renderer::Batch,
    transform::Transform,
};
use ordered_float::OrderedFloat;
use render::RenderObject;

pub struct InternalEntity {
    pub(crate) id: u64,
    pub transform: Transform,
    pub(crate) is_dead: bool,
    pub(crate) entity_type_id: TypeId,
    pub(crate) index: usize,
}

impl InternalEntity {
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
            entity_type_id: self.custom.type_id(),
            shaders_to_use: self.custom.shaders_to_use().clone(),
        }
    }
}

#[derive(Debug)]
pub enum ToGameState {
    InputMessage(Input),
    ScreenResize(i32, i32),
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Input {
    Key(glfw::Key, glfw::Action, glfw::Modifiers),
    Scroll(OrderedFloat<f64>),
    CursorPos(OrderedFloat<f64>, OrderedFloat<f64>),
}

pub struct GameState {
    pub(crate) entities: Vec<Entity>,
    pub camera: Camera,
    uid_incrementer: u64,
    /// fixed timestep in millis
    fixed_timestep: u64,
    fixed_delta_time: f32,
    to_app: Sender<ToApp>,
    inbox: Receiver<ToGameState>,
}

//self.entities.insert(id, entity_holder);
impl GameState {
    pub(crate) fn new(
        fixed_timestep: u64,
        to_app: Sender<ToApp>,
        inbox: Receiver<ToGameState>,
    ) -> Self {
        Self {
            entities: Vec::new(),
            camera: Camera::default(),
            uid_incrementer: 0,
            fixed_timestep,
            fixed_delta_time: (fixed_timestep as f32) / 1000.0,
            to_app,
            inbox,
        }
    }

    pub(crate) fn start_runtime(mut self, callbacks: &'static mut dyn GameCallbacks) {
        thread::spawn(move || {
            // game thread
            callbacks.start(&mut self);
            loop {
                thread::sleep(Duration::from_millis(self.fixed_timestep));
                let _ = self.to_app.send(InputRequest());
                let msgs: Vec<ToGameState> = self.inbox.try_iter().collect();
                for msg in msgs {
                    if let ToGameState::InputMessage(i) = msg {
                        callbacks.input(&mut self, &i);
                    } else {
                        self.hanlde_msg(&msg);
                    }
                }
                callbacks.update(&mut self);

                let robjs = self.fixed_update();
                let res = self
                    .to_app
                    .send(StartRender(Batch::new(robjs, self.camera.render_info())));
                if res.is_err() {
                    break;
                }
            }
        });
    }

    pub fn fixed_dt(&self) -> f32 {
        self.fixed_delta_time
    }

    fn fixed_update(&mut self) -> Arc<[RenderObject]> {
        let entities_len = self.entities.len();

        let dt = self.fixed_dt();
        let mut robjs = Vec::<RenderObject>::with_capacity(entities_len);
        for i in (0..entities_len).rev() {
            let mut e = self.entities.remove(i);
            e.custom.fixed_update(&mut e.internal, self, dt);
            if e.internal.is_dead {
                continue;
            }
            robjs.push(e.get_render_object());
            self.entities.push(e);
        }
        robjs.into()
    }

    fn hanlde_msg(&mut self, msg: &ToGameState) {
        use ToGameState as T;
        match *msg {
            T::ScreenResize(x, y) => {
                self.camera.screen_size = (x, y);
            }
            _ => {}
        }
    }

    pub fn new_entity<T: CustomEntity>(&mut self, custom: T) -> u64 {
        let uid = self.uid_incrementer;
        self.uid_incrementer += 1;

        let internal = InternalEntity {
            id: uid,
            transform: Transform::default(),
            is_dead: false,
            entity_type_id: custom.type_id(),
            index: self.entities.len(),
        };
        let mut entity = Entity {
            internal,
            custom: Box::new(custom),
        };
        entity.custom.start(&mut entity.internal, self);

        let tid = entity.custom.type_id();

        if !self.entities.iter().any(|x| x.custom.type_id() == tid) {
            let _ = self
                .to_app
                .send(CreateEntityRenderer(entity.custom.mesh_asset(), tid));
        }

        self.entities.push(entity);
        uid
    }

    pub fn kill_entity(&mut self, internal: &mut InternalEntity) {
        internal.is_dead = true;
        if !self
            .entities
            .iter()
            .any(|x| x.custom.type_id() == internal.entity_type_id)
        {
            let _ = self
                .to_app
                .send(RemoveEntityRenderer(internal.entity_type_id));
        }
    }
}
