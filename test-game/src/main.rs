use engine::{CustomEntity, EngineBuilder, GameState, InternalEntity, WORLD_AMBIENCE, WORLD_SCALE};
use lazy_static::lazy_static;
use render::{InnerObjectShader, MeshBuilder, ShaderInfo, SpecialUnis};

pub struct WireShader;
impl ShaderInfo for WireShader {
    fn name(&self) -> &str {
        "line"
    }

    fn set_special_uniforms(&self, e:&SpecialUnis, shader: &InnerObjectShader) {
        shader.set_vec3("camPos", e.cam_pos*WORLD_SCALE!());
    }
}

pub struct BaseShader;
impl ShaderInfo for BaseShader {
    fn name(&self) -> &str {
        "diffuse"
    }

    fn set_special_uniforms(&self, e:&SpecialUnis, shader: &InnerObjectShader) {
        shader.set_float("ambientStrength", WORLD_AMBIENCE!());
        shader.set_vec3("lightPos", e.cam_pos*WORLD_SCALE!());
    }
}

struct Cube {
    timer: f32,
}

lazy_static!{
    static ref CUBE_MESH: render::Mesh = MeshBuilder::builder(vec![
            -5.0, -5.0, -5.0, 5.0, -5.0, -5.0, 5.0, 5.0, -5.0, -5.0, 5.0, -5.0, -5.0, -5.0, 5.0,
            5.0, -5.0, 5.0, 5.0, 5.0, 5.0, -5.0, 5.0, 5.0,
        ])
        .with_colors(vec![
            1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0,
            0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0,
        ])
        .with_indices(vec![
            0, 1, 2, 0, 2, 3, // front
            4, 5, 6, 4, 6, 7, // back
            0, 4, 7, 0, 7, 3, // left
            1, 5, 6, 1, 6, 2, // right
            0, 1, 5, 0, 5, 4, // bottom
            3, 2, 6, 3, 6, 7, // top
        ])
        .build();

    static ref CUBE_SHADERS: Vec<u8> = vec![0, 1];
}


impl CustomEntity for Cube {
    fn start(&mut self, _: &mut InternalEntity, _: &mut GameState) {}

    fn fixed_update(&mut self, a: &mut InternalEntity, _: &mut GameState, dt: f32) {
        a.transform.add_position(4.0 * engine::FRONT * dt);

        if self.timer > 1.3 {
            a.kill();
        }

        self.timer += dt;
    }

    fn mesh(&self) -> &render::Mesh {
        &CUBE_MESH
    }

    fn shaders_to_use(&self) -> Vec<u8> {
        CUBE_SHADERS.clone()
    }
}

fn main() {
    let mut engine = EngineBuilder::builder()
        .res(1000, 1000)
        .shaders_path("/home/mutoxicated/Desktop/Software/gooberverse2/test-game/resources/shaders/")
        .shader_info(vec![Box::new(WireShader{}), Box::new(BaseShader{})])
        .with_fixed_timestep(50)
        .build();
    engine.app.set_bg_color(0.0, 0.1, 0.1);
    engine.game_state.new_entity(Cube { timer: 0f32 });
    engine.run(|_| {});
}
