mod camera;
mod constants;
mod transform;
mod utils;
mod renderer;
mod game_state;
mod app;

pub use game_state::GameState;

use std::{
    thread::{self}
};

use gl::{
    BLEND, DEPTH_TEST, MULTISAMPLE, ONE_MINUS_SRC_ALPHA,
    SRC_ALPHA,
};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use render::{ObjectShader, ShaderInfo};

use crate::{app::{App, ToApp}, renderer::{Renderer, ToRenderer}};

/// Note: game_state does not include the custom entity itself (i.e. the `self`)
pub trait CustomEntity: Send + 'static {
    fn start(&mut self, game_state: &mut GameState);
    fn update(&mut self, game_state: &mut GameState, delta_time:f32);
    fn mesh(&self) -> render::Mesh;
}

pub struct EngineBuilder {
    width: u32,
    height: u32,
    shaders_path: String,
    shader_info: Option<Vec<Box<dyn ShaderInfo + Send + 'static>>>,
}

impl EngineBuilder {
    fn glfw_constructor() -> Glfw {
        let mut glfw = glfw::init(|e, desc| {
            exit!("GLFW failed. \n* Error: '{}'\n* Description: '{}'", e, desc);
        })
        .unwrap_or_else(|a| {
            exit!("GLFW initialization failed. Message: {a}");
        });
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::Samples(Some(4)));
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        glfw
    }

    fn window_event_constructor(
        glfw: &mut Glfw,
        width: u32,
        height: u32,
    ) -> (PWindow, GlfwReceiver<(f64, WindowEvent)>) {
        let (mut window, events) = glfw
            .create_window(
                width,
                height,
                "Powered by GooberVerse",
                glfw::WindowMode::Windowed,
            )
            .unwrap_or_else(|| {
                exit!("Failed to create a window.");
            });

        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_scroll_polling(true);
        window.set_cursor_mode(glfw::CursorMode::Normal);

        gl::load_with(|symbol| {
            window.get_proc_address(symbol).unwrap_or_else(|| {
                exit!("Couldn't load opengl functions (failed at '{}')", symbol);
            }) as *const _
        });
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::Enable(DEPTH_TEST);
            gl::Enable(MULTISAMPLE);

            gl::BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
            gl::Enable(BLEND);
        }

        (window, events)
    }

    pub fn builder() -> Self {
        Self {
            width: 0,
            height: 0,
            shaders_path: String::new(),
            shader_info: None,
        }
    }

    pub fn res(mut self, w: u32, h: u32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    pub fn shaders_path(mut self, v: String) -> Self {
        self.shaders_path = v;
        self
    }

    pub fn shader_info(mut self, v: Vec<Box<dyn ShaderInfo + Send + 'static>>) -> Self {
        self.shader_info = Some(v);
        self
    }

    pub fn build(self) -> Engine
    {
        assert_ne!(self.width, 0);
        assert_ne!(self.height, 0);
        assert_ne!(self.shaders_path, "".to_owned());
        assert!(self.shader_info.is_some());

        let mut glfw = Self::glfw_constructor();
        let (window, events) =
            Self::window_event_constructor(&mut glfw, self.width, self.height);
        let mut obj_shaders: Vec<ObjectShader> = Vec::new();
        for v in self.shader_info.unwrap() {
            obj_shaders.push(ObjectShader::new(v, self.shaders_path.to_owned()));
        }
        let (sender, r) = std::sync::mpsc::channel::<ToRenderer>();
        let (sender2, r2) = std::sync::mpsc::channel::<ToApp>();

        let app = App {
            glfw,
            window,
            events,
            to_renderer: sender,
            inbox: r2,
            game_state: GameState::default()
        };
        let renderer = Renderer::new(obj_shaders, r, sender2);

        Engine { renderer, app }
    }
}


pub struct Engine {
    renderer: Renderer,
    pub app: App
}

impl Engine {
    pub fn run(self, func: impl FnMut(&mut App)) {
        thread::spawn(move || self.renderer);
        self.app.start_runtime(func);
    }
}