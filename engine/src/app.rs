use glfw::{Action, Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use render::mesh::MeshAsset;
use std::any::TypeId;
use std::assert_matches;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use crate::{
    AppCallbacks,
    game_state::{
        Input,
        ToGameState::{self, InputMessage, ScreenResize},
    },
    renderer::{Batch, Renderer},
};

pub enum ToApp {
    StartRender(Batch),
    CreateEntityRenderer(MeshAsset, TypeId),
    RemoveEntityRenderer(TypeId),
    InputRequest(),
}

pub struct AppBuilder {
    glfw: Option<Glfw>,
    window: Option<PWindow>,
    events: Option<GlfwReceiver<(f64, WindowEvent)>>,
    inbox: Option<Receiver<ToApp>>,
    to_game_state: Option<Sender<ToGameState>>,
    renderer: Option<Renderer>,
}

impl AppBuilder {
    pub fn builder() -> Self {
        Self {
            glfw: None,
            window: None,
            events: None,
            inbox: None,
            to_game_state: None,
            renderer: None,
        }
    }

    pub fn glfw_window_events(
        mut self,
        glfw: Glfw,
        window: PWindow,
        events: GlfwReceiver<(f64, WindowEvent)>,
    ) -> Self {
        self.glfw = Some(glfw);
        self.window = Some(window);
        self.events = Some(events);
        self
    }
    pub fn inbox(mut self, inbox: Receiver<ToApp>) -> Self {
        self.inbox = Some(inbox);
        self
    }

    pub fn renderer(mut self, renderer: Renderer) -> Self {
        self.renderer = Some(renderer);
        self
    }

    pub fn to_game_state(mut self, to_game_state: Sender<ToGameState>) -> Self {
        self.to_game_state = Some(to_game_state);
        self
    }

    pub fn build(self) -> App {
        assert!(self.glfw.is_some());
        assert!(self.inbox.is_some());
        assert!(self.to_game_state.is_some());
        assert!(self.renderer.is_some());
        assert!(self.window.is_some());
        assert!(self.events.is_some());

        App {
            inbox: self.inbox.unwrap(),
            to_game_state: self.to_game_state.unwrap(),
            renderer: self.renderer.unwrap(),
            window: self.window.unwrap(),
            events: self.events.unwrap(),
            glfw: self.glfw.unwrap(),
            game_inputs: HashMap::new(),
        }
    }
}

// NOTE: The order of some fields is VERY important: glfw NEEDS to be
// the last field, because it has to always get dropped last.
pub struct App {
    inbox: Receiver<ToApp>,
    to_game_state: Sender<ToGameState>,
    renderer: Renderer,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
    glfw: Glfw,
    game_inputs: HashMap<Input, ()>,
}

impl App {
    pub fn start_runtime(mut self, callbacks: &'static dyn AppCallbacks) {
        callbacks.start(&mut self);
        let (x, y) = self.window.get_size();
        let _ = self.to_game_state.send(ScreenResize(x, y));

        while !self.window.should_close() {
            self.handle_events();
            callbacks.update(&mut self);

            self.glfw.poll_events();
            self.check_inbox();
        }
    }

    pub fn handle_events(&mut self) {
        use glfw::WindowEvent as W;

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                W::FramebufferSize(w, h) => unsafe {
                    gl::Viewport(0, 0, w, h);
                    let _ = self.to_game_state.send(ScreenResize(w, h));
                },
                W::Key(glfw::Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true);
                }
                W::Key(glfw::Key::Tab, _, Action::Press, _) => {
                    let mode = if self.window.get_cursor_mode() == glfw::CursorMode::Disabled {
                        glfw::CursorMode::Normal
                    } else {
                        glfw::CursorMode::Disabled
                    };
                    self.window.set_cursor_mode(mode);
                }
                W::Scroll(_, dy) => {
                    let _ = self
                        .to_game_state
                        .send(InputMessage(Input::Scroll(dy.into())));
                }
                W::CursorPos(x, y) => {
                    if !self
                        .game_inputs
                        .iter()
                        .any(|(a, _)| matches!(a, Input::CursorPos(_, _)))
                    {
                        self.game_inputs
                            .insert(Input::CursorPos(x.into(), y.into()), ());
                    }
                }
                _ => {}
            }
        }
    }

    pub fn check_inbox(&mut self) {
        use ToApp as T;

        let msgs: Vec<ToApp> = self.inbox.try_iter().collect();
        let (renders, others): (Vec<&T>, Vec<&T>) =
            msgs.iter().partition(|a| matches!(a, T::StartRender(_)));
        for msg in &others {
            match *msg {
                T::CreateEntityRenderer(mesh, tid) => {
                    let res = mesh.load_mesh();
                    if let Ok(x) = res {
                        self.renderer.new_entity_renderer(&x, tid.clone());
                    } else {
                        println!("{:?}", res.unwrap_err())
                    }
                }
                T::RemoveEntityRenderer(eid) => {
                    self.renderer.remove_entity_renderer(*eid);
                }
                T::InputRequest() => {
                    self.game_inputs.drain().for_each(|(k, _)| {
                        let _ = self.to_game_state.send(InputMessage(k));
                    });
                }
                _ => {}
            }
        }
        if let Some(r) = renders.into_iter().last()
            && let T::StartRender(batch) = r
        {
            self.renderer.render(batch);
            self.window.swap_buffers();
        }
    }

    pub fn window(&self) -> &PWindow {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut PWindow {
        &mut self.window
    }

    pub fn set_bg_color(&self, r: f32, g: f32, b: f32) {
        unsafe {
            gl::ClearColor(r, g, b, 1.0);
        }
    }

    pub fn add_game_state_input(&mut self, t: ToGameState) {
        assert_matches!(t, ToGameState::InputMessage(_));
        if let InputMessage(a) = t
            && !self.game_inputs.contains_key(&a)
        {
            self.game_inputs.insert(a, ());
        }
    }
}
