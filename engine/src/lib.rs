mod app;
mod camera;
mod constants;
pub mod game_state;
mod renderer;
mod transform;
mod utils;

use std::rc::Rc;
use std::sync::Arc;

pub use app::App;
pub use game_state::GameState;
pub use game_state::Input;
pub use game_state::InternalEntity;
pub use game_state::ToGameState;
pub use camera::Camera;

use glam::Vec3;

pub const FRONT: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 1.0,
};
pub const UP: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
pub const RIGHT: Vec3 = Vec3 {
    x: 1.0,
    y: 0.0,
    z: 0.0,
};

use gl::{BLEND, DEPTH_TEST, MULTISAMPLE, ONE_MINUS_SRC_ALPHA, SRC_ALPHA};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use render::{ObjectShader, ShaderInfo};

use crate::{app::ToApp, renderer::Renderer};

/// Note: game_state does not include the custom entity itself (i.e. the `self`)
pub trait CustomEntity: Send + 'static {
    fn start(&mut self, inner: &mut InternalEntity, game_state: &mut GameState);
    fn fixed_update(
        &mut self,
        inner: &mut InternalEntity,
        game_state: &mut GameState,
        fixed_dt: f32,
    );
    fn mesh(&self) -> &'static render::Mesh;
    fn shaders_to_use(&self) -> &'static Vec<u8>;
}

pub const SHADERS_PATH: &str = "./assets/shaders/";
pub const MESHES_PATH: &str = "./assets/meshes/";

pub struct EngineBuilder {
    width: u32,
    height: u32,
    shader_info: Option<Vec<Box<dyn ShaderInfo>>>,
    fixed_step_millis: u64,
    app_callbacks: Option<&'static dyn AppCallbacks>,
    game_callbacks: Option<&'static mut dyn GameCallbacks>,
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
            shader_info: None,
            fixed_step_millis: 100,
            app_callbacks: None,
            game_callbacks: None,
        }
    }

    pub fn res(mut self, w: u32, h: u32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    pub fn shader_info(mut self, v: Vec<Box<dyn ShaderInfo>>) -> Self {
        self.shader_info = Some(v);
        self
    }

    pub fn game_callbacks(mut self, game_calllbacks: &'static mut dyn GameCallbacks) -> Self {
        self.game_callbacks = game_calllbacks.into();
        self
    }

    pub fn app_callbacks(mut self, app_callbacks: &'static dyn AppCallbacks) -> Self {
        self.app_callbacks = app_callbacks.into();
        self
    }

    pub fn with_fixed_timestep(mut self, timestep_sec: u64) -> Self {
        self.fixed_step_millis = timestep_sec;
        self
    }

    pub fn build(self) -> Engine {
        assert_ne!(self.width, 0);
        assert_ne!(self.height, 0);
        assert!(self.shader_info.is_some());
        assert!(self.game_callbacks.is_some());
        assert!(self.app_callbacks.is_some());

        let mut glfw = Self::glfw_constructor();
        let (window, events) = Self::window_event_constructor(&mut glfw, self.width, self.height);
        let mut obj_shaders: Vec<ObjectShader> = Vec::new();
        for v in self.shader_info.unwrap() {
            obj_shaders.push(ObjectShader::new(v, SHADERS_PATH.to_owned()));
        }

        let (sender, r) = std::sync::mpsc::channel::<ToGameState>();
        let (sender2, r2) = std::sync::mpsc::channel::<ToApp>();

        let renderer = Renderer::new(obj_shaders, (self.fixed_step_millis as f32)/1000.0);
        let app = App {
            glfw,
            window,
            events,
            renderer,
            inbox: r2,
            to_game_state: sender,
        };
        let game_state = GameState::new(self.fixed_step_millis, sender2, r);

        Engine {
            app,
            game_state,
            app_callbacks: self.app_callbacks.unwrap(),
            game_callbacks: self.game_callbacks.unwrap(),
        }
    }
}

pub trait GameCallbacks: Send + Sync + 'static {
    fn start(&mut self, game: &mut GameState);
    fn update(&mut self, game: &mut GameState);
    fn input(&mut self, game: &mut GameState, input: &Input);
}

pub trait AppCallbacks {
    fn start(&self, app: &mut App);
    fn update(&self, app: &mut App);
}

pub struct Engine {
    pub app: App,
    game_state: GameState,
    game_callbacks: &'static mut dyn GameCallbacks,
    app_callbacks: &'static dyn AppCallbacks,
}

impl Engine {
    pub fn run(self) {
        self.game_state.start_runtime(self.game_callbacks);
        self.app.start_runtime(self.app_callbacks);
    }
}
