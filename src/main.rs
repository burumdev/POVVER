mod app_state;
mod economy;
mod environment;
mod ui_controller;
mod utils_random;
mod utils_traits;
mod utils_data;
mod logger;

mod simulation;
use simulation::Simulation;

fn main() {
    let mut sim = Simulation::new();

    sim.run();
}
