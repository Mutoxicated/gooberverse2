use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

use crate::{
    CustomEntity, GameCallbacks, app::ToApp::{self, CreateEntityRenderer, RemoveEntityRenderer, StartRender}, camera::Camera, game_state::ToGameState::{InputMessage, ScreenResize}, renderer::Batch, transform::Transform
};
use glfw::Key;
use render::RenderObject;

pub struct InternalEntity {
    pub(crate) id: u64,
    pub transform: Transform,
    pub(crate) is_dead: bool,
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
            entity_id: self.internal.id,
            shaders_to_use: self.custom.shaders_to_use().clone(),
        }
    }
}

pub enum ToGameState {
    InputMessage(Input),
    ScreenResize(i32, i32)
}

pub enum Input {
    Key(glfw::Key, glfw::Action, glfw::Modifiers),
    Scroll(f64),
    CursorPos(f64, f64),
}

pub struct GameState {
    pub(crate) entities: Vec<Entity>,
    pub camera: Camera,
    uid_incrementer: u64,
    /// fixed timestep in seconds
    fixed_timestep: u64,
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

                let msgs: Vec<ToGameState> = self.inbox.try_iter().collect();
                for msg in msgs {
                    if let InputMessage(i) = msg {
                        callbacks.input(&mut self, &i);
                    }else {
                        self.hanlde_msg(&msg);
                    }
                }

                callbacks.update(&mut self);
                let robjs = self.fixed_update();
                let res = self
                    .to_app
                    .send(StartRender(Batch::new(robjs, self.camera.clone())));
                if res.is_err() {
                    break;
                }
            }
        });
    }

    pub fn fixed_dt(&self) -> f32 {
        (self.fixed_timestep as f32) / 1000.0
    }

    fn fixed_update(&mut self) -> Vec<RenderObject> {
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
            a.custom
                .fixed_update(&mut a.internal, self, self.fixed_dt());
            if a.internal.is_dead {
                i -= 1;
                continue;
            }
            if !a.custom.mesh().is_invalid() {
                robjs.push(a.get_render_object());
            }
            self.entities.push(a);
            i -= 1;
        }
        robjs
    }

    fn hanlde_msg(&mut self, msg: &ToGameState) {
        use ToGameState::*;
        match *msg {
            ScreenResize(x, y) => {
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
        };
        let mut entity = Entity {
            internal,
            custom: Box::new(custom),
        };
        let _ = self.to_app.send(CreateEntityRenderer(
            entity.custom.mesh(),
            entity.internal.id,
        ));
        entity.custom.start(&mut entity.internal, self);
        self.entities.push(entity);

        uid
    }

    pub fn kill_entity(&mut self, internal: &mut InternalEntity) {
        internal.is_dead = true;
        let _ = self.to_app.send(RemoveEntityRenderer(internal.id));
    }
}
