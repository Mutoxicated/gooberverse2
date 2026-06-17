use std::{iter::Filter, slice::Iter, sync::mpsc::{Receiver, Sender}};

use glfw::{Action, Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use render::Mesh;

use crate::{
    AppCallbacks,
    game_state::{Input, ToGameState::{self, InputMessage, ScreenResize}},
    renderer::{Batch, Renderer},
};

pub enum ToApp {
    StartRender(Batch),
    CreateEntityRenderer(&'static Mesh, u64),
    RemoveEntityRenderer(u64),
}

// NOTE: The order of some fields is VERY important: glfw NEEDS to be
// the last field, because it has to always get dropped last.
pub struct App {
    pub(crate) inbox: Receiver<ToApp>,
    pub(crate) to_game_state: Sender<ToGameState>,
    pub(crate) renderer: Renderer,
    pub(crate) window: PWindow,
    pub(crate) events: GlfwReceiver<(f64, WindowEvent)>,
    pub(crate) glfw: Glfw,
}

impl App {
    pub fn start_runtime(mut self, callbacks: &'static dyn AppCallbacks) {
        callbacks.start(&mut self);
        let (x, y) = self.window.get_size();
        let _ = self.to_game_state.send(ScreenResize(x,y));

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
                    let _ = self.to_game_state.send(InputMessage(Input::Scroll(dy)));
                }
                W::CursorPos(x, y) => {
                    let _ = self.to_game_state.send(InputMessage(Input::CursorPos(x, y)));
                }
                _ => {}
            }
        }
    }

    pub fn check_inbox(&mut self) {
        use ToApp as T;

        let msgs: Vec<ToApp> = self.inbox.try_iter().collect();
        let (renders, others): (Vec<&T>, Vec<&T>) = msgs.iter().partition(|a| matches!(a, T::StartRender(_)));
        for msg in &others {
            match *msg {
                T::CreateEntityRenderer(mesh, eid) => {
                    self.renderer.new_entity_renderer(mesh, *eid);
                }
                T::RemoveEntityRenderer(eid) => {
                    self.renderer.remove_entity_renderer(*eid);
                }
                _ => {}
            }
        }
        if let Some(r) = renders.into_iter().last() && let T::StartRender(batch) = r {
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

    pub fn send_to_game_state(&mut self, t: ToGameState) {
        let _ = self.to_game_state.send(t);
    }
}
