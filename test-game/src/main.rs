use engine::{Engine, EngineBuilder};

fn main() {
    let eb = EngineBuilder::new(1000, 1000)("../resources", Vec::new());
    Engine::start(eb, |api| {
        api.new_entity();
    });
}
