#![deny(unsafe_op_in_unsafe_fn)]

use std::any::TypeId;

use engine::{
    AppCallbacks, Camera, CustomEntity, EngineBuilder, GameCallbacks, GameState,
    Input::{self, CursorPos},
    InternalEntity, WORLD_AMBIENCE, WORLD_SCALE,
};
use glam::{DVec2, Vec3};
use glfw::Action;
use lazy_static::lazy_static;
use render::{
    InnerObjectShader, ShaderInfo, SpecialUnis,
    mesh::{ExtraOptions, MeshAsset, MeshFileType, WireType},
};

pub struct WireShader;
impl ShaderInfo for WireShader {
    fn name(&self) -> &str {
        "line"
    }

    fn set_special_uniforms(&self, e: &SpecialUnis, shader: &InnerObjectShader) {
        shader.set_vec3("camPos", e.cam_pos * WORLD_SCALE!());
    }
}

pub struct BaseShader;
impl ShaderInfo for BaseShader {
    fn name(&self) -> &str {
        "diffuse"
    }

    fn set_special_uniforms(&self, e: &SpecialUnis, shader: &InnerObjectShader) {
        shader.set_float("ambientStrength", WORLD_AMBIENCE!());
        shader.set_vec3("lightPos", e.cam_pos * WORLD_SCALE!());
    }
}

#[derive(Default)]
struct Cube {
    timer: f32,
}

lazy_static! {
    static ref CUBE_SHADERS: Vec<u8> = vec![0, 1];
}

impl CustomEntity for Cube {
    fn start(&mut self, i: &mut InternalEntity, _: &mut GameState) {
        i.transform.set_position(Vec3::new(0.0, 0.0, 10.0));
    }

    fn fixed_update(&mut self, a: &mut InternalEntity, state: &mut GameState, dt: f32) {
        //a.transform.add_position(4.0 * engine::FRONT * dt);
        self.timer += dt;
        a.transform.add_position(engine::RIGHT * dt);
        // if self.timer >= 3.2 {
        //     state.kill_entity(a);
        // }
    }

    fn mesh_asset(&self) -> MeshAsset {
        MeshAsset::new(
            "assets/meshes/cube",
            MeshFileType::GLTF,
            ExtraOptions::BakeWireframe(WireType::Quad),
        )
    }

    fn shaders_to_use(&self) -> &'static Vec<u8> {
        &CUBE_SHADERS
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Cube>()
    }
}

#[derive(Default)]
pub struct Game {
    prev_mouse_pos: DVec2,
    timer: i32,
    cam_speed: f32,
}
impl GameCallbacks for Game {
    fn start(&mut self, state: &mut GameState) {
        state.new_entity(Cube::default());
    }

    fn update(&mut self, _state: &mut GameState) {
        self.timer += 1;
        // if self.timer % 32 == 0 {
        //     state.new_entity(Cube::default());
        // }
    }

    fn input(&mut self, state: &mut GameState, input: &Input) {
        match *input {
            Input::Key(glfw::Key::W, glfw::Action::Repeat, _) => {
                state.camera.position += state.camera.front * self.cam_speed * state.fixed_dt();
            }
            Input::Key(glfw::Key::S, glfw::Action::Repeat, _) => {
                state.camera.position -= state.camera.front * self.cam_speed * state.fixed_dt();
            }
            Input::Key(glfw::Key::A, glfw::Action::Repeat, _) => {
                state.camera.position += state.camera.front.cross(engine::UP).normalize()
                    * self.cam_speed
                    * state.fixed_dt();
            }
            Input::Key(glfw::Key::D, glfw::Action::Repeat, _) => {
                state.camera.position -= state.camera.front.cross(engine::UP).normalize()
                    * self.cam_speed
                    * state.fixed_dt();
            }
            Input::Scroll(dy) => {
                let a = dy.0 as f32 * state.fixed_dt() * 180.0;
                state.camera.fov = (state.camera.fov - a).clamp(30.0, 170.0);
            }
            Input::CursorPos(x, y) => {
                //println!("[GameState] CursorPos");
                let new_mouse_position = DVec2::new(x.0, y.0);
                if self.prev_mouse_pos == DVec2::splat(-1.0) {
                    self.prev_mouse_pos = new_mouse_position;
                    return;
                }
                let data = new_mouse_position - self.prev_mouse_pos;
                self.prev_mouse_pos = new_mouse_position;
                //println!("dx: {dx}, dy: {dy}");
                state
                    .camera
                    .rotate_from_raw_cursor_data(data.as_vec2(), state.fixed_dt());
            }
            _ => {}
        }
    }
}

pub struct App;
impl AppCallbacks for App {
    fn start(&self, _app: &mut engine::App) {}

    fn update(&self, app: &mut engine::App) {
        if app.window().get_key(glfw::Key::W) == Action::Press {
            app.add_game_state_input(engine::ToGameState::InputMessage(Input::Key(
                glfw::Key::W,
                glfw::Action::Repeat,
                glfw::Modifiers::empty(),
            )));
        }
        if app.window().get_key(glfw::Key::S) == Action::Press {
            app.add_game_state_input(engine::ToGameState::InputMessage(Input::Key(
                glfw::Key::S,
                glfw::Action::Repeat,
                glfw::Modifiers::empty(),
            )));
        }
        if app.window().get_key(glfw::Key::A) == Action::Press {
            app.add_game_state_input(engine::ToGameState::InputMessage(Input::Key(
                glfw::Key::A,
                glfw::Action::Repeat,
                glfw::Modifiers::empty(),
            )));
        }
        if app.window().get_key(glfw::Key::D) == Action::Press {
            app.add_game_state_input(engine::ToGameState::InputMessage(Input::Key(
                glfw::Key::D,
                glfw::Action::Repeat,
                glfw::Modifiers::empty(),
            )));
        }
    }
}

pub static mut GAME: Game = Game {
    prev_mouse_pos: DVec2 { x: -1.0, y: -1.0 },
    timer: 0i32,
    cam_speed: 8.4,
};

fn main() {
    #[allow(static_mut_refs)]
    let engine = EngineBuilder::builder()
        .res(1000, 1000)
        .shader_info(vec![Box::new(WireShader {}), Box::new(BaseShader {})])
        .with_fixed_timestep(17)
        .app_callbacks(&App)
        .game_callbacks(unsafe { &mut GAME })
        .build();
    engine.app.set_bg_color(0.0, 0.1, 0.1);
    engine.run();
}
