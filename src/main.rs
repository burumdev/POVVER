mod economy;
mod environment;
mod months;
mod simulation;
mod speed;
mod timer;
mod ui_controller;
mod utils_random;

use simulation::Simulation;

fn main() {
    let mut sim = Simulation::new();

    sim.run();
}
