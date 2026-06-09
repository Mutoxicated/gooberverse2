mod utils;
mod runtime;

use std::{thread::{self, ScopedJoinHandle}};

use gl::{BLEND, DEPTH_TEST, MULTISAMPLE, ONE_MINUS_SRC_ALPHA, SRC_ALPHA};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};

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

    fn render_loop(window: &mut PWindow) {
        while !window.should_close() {
            window.swap_buffers();
            println!("render");
        }
    }

    fn game_loop(mut glfw:Glfw, render_thread:ScopedJoinHandle<'_, ()>) {
        let mut delta_time = 0f32;
        let mut prev_time = glfw.get_time();
        while !render_thread.is_finished() {
            let time = glfw.get_time();
            delta_time = (time-prev_time) as f32;
            prev_time = time;
            println!("game");
            
            glfw.poll_events();
        }
    }

    pub fn start(width:u32, height:u32) {
        thread::scope(|s| {
            let _ = s.spawn(|| {
                let mut glfw = Self::glfw_constructor();
                let (mut window, events) = Self::window_event_constructor(&mut glfw, width, height);

                let render_thread = s.spawn(move || Self::render_loop(&mut window));

                Self::game_loop(glfw, render_thread);
            });
        });
    }
}