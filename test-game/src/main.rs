use engine::{CustomEntity, EngineBuilder, GameState};

struct Cube {

}

impl CustomEntity for Cube {
    fn start(&mut self, game_state: &mut GameState) {
        todo!()
    }

    fn update(&mut self, game_state: &mut GameState, delta_time:f32) {
        todo!()
    }

    fn mesh(&self) -> render::Mesh {
        todo!()
    }
}

fn main() {
    let mut engine = EngineBuilder::builder()
        .res(1000, 1000)
        .shaders_path("../resources".to_owned())
        .shader_info(vec![])
        .build();

    engine.app.game_state.new_entity(Cube{});
    engine.run(|app| {
    });
}
