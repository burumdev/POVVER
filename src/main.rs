mod environment;
mod entities;
mod months;
mod simulation;
mod speed;
mod timer;
mod utils;

use simulation::Simulation;

fn main() {
    let mut sim = Simulation::new();

    sim.run();
}
