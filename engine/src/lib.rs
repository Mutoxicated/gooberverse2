mod utils;
mod runtime;
mod transform;
mod camera;
mod constants;

use std::{any::Any, sync::mpsc::{Receiver, Sender}, thread::{self}};

use gl::{BLEND, COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, DEPTH_TEST, MULTISAMPLE, ONE_MINUS_SRC_ALPHA, SRC_ALPHA};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use render::{ObjectShader, ShaderInfo};

use crate::{camera::Camera, transform::Transform};

pub struct EngineBuilder {
    width: u32,
    height: u32,
    object_shaders: Vec<ObjectShader>
}

impl EngineBuilder {
    pub fn new(width: u32, height: u32) -> Box<dyn FnOnce(&str, Vec<Box<dyn ShaderInfo + Send>>) -> EngineBuilder> {
        Box::new(move |shaders_path:&str, shader_info: Vec<Box<dyn ShaderInfo + Send>>| {
            let mut object_shaders:Vec<ObjectShader> = Vec::new();
            for v in shader_info {
                object_shaders.push(ObjectShader::new(v, shaders_path.to_owned()));
            }
            EngineBuilder { width, height, object_shaders }
        })
    }
}

pub struct EntityInternal {
    pub id: u64,
    pub drawer: render::EntityRenderer,
    pub transform: Transform,
}

pub trait CustomEntity {
    fn start(&mut self, entity: &mut EntityInternal);
    fn update(&mut self, entity: &mut EntityInternal, delta_time: f32);
    fn mesh(&self) -> render::Mesh;
    fn shaders_to_use(&self) -> Vec<u8>;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct Entity {
    pub inner: EntityInternal,
    pub interface: Box<dyn CustomEntity + Send>
}

enum ToRenderer {
    Render(Vec<Entity>, Camera)
}

enum ToCore {
    RenderDone,
}

pub struct API<'a> {
    core: &'a mut Core
}

impl<'a> API<'a> {
    pub fn new_entity(&self) -> () {
    }
}

struct Renderer {
    shaders: Vec<ObjectShader>,
    from_core: Receiver<ToRenderer>
}

impl Renderer {
    fn runtime_loop(mut self) {
        use ToRenderer::*;

        while let Ok(msg) = self.from_core.recv() {
            match msg {
                Render(entities, camera) => {
                    self.render(entities, camera);
                }
            }
        }
    }

    fn render(&mut self, entities: Vec<Entity>, camera: Camera) {
        unsafe { gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT); }

        let (proj, view) = (camera.proj_matrix(), camera.view_matrix());
        for os in &self.shaders {
            os.shader.activate();
            os.shader.set_mat4("proj", proj);
            os.shader.set_mat4("view", view);
            os.shader.set_float("scale", WORLD_SCALE!());
        }
        let special_unis = render::SpecialUnis{cam_pos:camera.position};
        for go in entities {
            let entity = &go.inner;
            for shader_index in go.interface.shaders_to_use() {
                let os = &self.shaders[shader_index as usize];
                os.shader.activate();
                os.shader.set_mat4("model", entity.transform.model_matrix());
                os.info.set_special_uniforms(&special_unis, &os.shader);
                entity.drawer.draw();
            }
        }
        get_gl_error!();
    }
}

pub struct Core {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
    to_renderer: Sender<ToRenderer>
}

impl Core {
    fn start_runtime<F>(mut self, mut custom_runtime: F)
    where F: FnMut(API) 
    {
        let mut delta_time = 0f32;
        let mut prev_time = self.glfw.get_time();
        while !self.window.should_close() {
            let time = self.glfw.get_time();
            delta_time = (time-prev_time) as f32;
            prev_time = time;

            let api = API { core: &mut self };
            custom_runtime(api);

            self.glfw.poll_events();
        }
    }
}

pub struct Engine;

impl Engine {
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

    fn window_event_constructor(glfw:&mut Glfw, width:u32, height:u32) -> (PWindow, GlfwReceiver<(f64, WindowEvent)>) {
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

    pub fn start<F>(eb: EngineBuilder, custom_runtime: F) 
    where F: FnMut(API)
    {
        thread::scope(|s| {
            let mut glfw = Self::glfw_constructor();
            let (window, events) = Self::window_event_constructor(&mut glfw, eb.width, eb.height);
            let (sender, r) = std::sync::mpsc::channel::<ToRenderer>();
            let core = Core {
                glfw, 
                window, 
                events,
                to_renderer: sender
            };
            let render = Renderer { 
                shaders: eb.object_shaders,
                from_core: r
            };
            
            s.spawn(move || render.runtime_loop());
            core.start_runtime(custom_runtime);
        });
    }
}