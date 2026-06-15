use std::sync::mpsc::{Receiver, Sender};

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
        self.to_game_state.send(ScreenResize(x,y));

        let mut delta_time: f32;
        let mut prev_time = self.glfw.get_time();
        while !self.window.should_close() {
            // app thread
            let time = self.glfw.get_time();
            delta_time = (time - prev_time) as f32;
            prev_time = time;

            self.handle_events();
            callbacks.update(&mut self);

            self.glfw.poll_events();
            self.check_inbox();
        }
    }

    pub fn handle_events(&mut self) {
        use glfw::WindowEvent::*;

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                FramebufferSize(w, h) => unsafe {
                    gl::Viewport(0, 0, w, h);
                    let _ = self.to_game_state.send(ScreenResize(w, h));
                },
                Key(glfw::Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true);
                }
                Key(glfw::Key::Tab, _, Action::Press, _) => {
                    let mode = if self.window.get_cursor_mode() == glfw::CursorMode::Disabled {
                        glfw::CursorMode::Normal
                    } else {
                        glfw::CursorMode::Disabled
                    };
                    self.window.set_cursor_mode(mode);
                }
                Key(a, _, b, c) => {
                    let _ = self.to_game_state.send(InputMessage(Input::Key(a, b, c)));
                }
                Scroll(_, dy) => {
                    let _ = self.to_game_state.send(InputMessage(Input::Scroll(dy)));
                }
                CursorPos(x, y) => {
                    let _ = self.to_game_state.send(InputMessage(Input::CursorPos(x, y)));
                }
                _ => {}
            }
        }
    }

    pub fn check_inbox(&mut self) {
        use ToApp::*;

        let result = self.inbox.try_recv();
        if result.is_err() {
            return;
        }
        match result.unwrap() {
            StartRender(batch) => {
                println!("[App] StartRender");
                self.renderer.render(batch);
                self.window.swap_buffers();
            }
            CreateEntityRenderer(mesh, eid) => {
                println!("[App] CreateEntityRenderer");
                self.renderer.new_entity_renderer(mesh, eid);
            }
            RemoveEntityRenderer(eid) => {
                println!("[App] RemoveEntityRenderer");
                self.renderer.remove_entity_renderer(eid);
            }
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
}
