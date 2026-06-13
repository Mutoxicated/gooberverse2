use std::sync::mpsc::{Receiver, Sender};

use glfw::{Glfw, GlfwReceiver, PWindow, WindowEvent};

use crate::{CustomEntity, game_state::{self, GameState}, renderer::ToRenderer};

pub enum ToApp {
    RenderDone,
}

pub struct App {
    pub glfw: Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,

    pub to_renderer: Sender<ToRenderer>,
    pub inbox: Receiver<ToApp>,

    pub game_state: GameState
}

impl App {
    pub fn start_runtime(mut self, mut func: impl FnMut(&mut App))
    {
        //use ToRenderer::*;func
        let mut delta_time = 0f32;
        let mut prev_time = self.glfw.get_time();
        while !self.window.should_close() {
            let time = self.glfw.get_time();
            delta_time = (time - prev_time) as f32;
            prev_time = time;

            self.game_state.update(delta_time);
            func(&mut self);

            self.glfw.poll_events();
        }
    }
}