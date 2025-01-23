mod environment;
mod environment_types;
mod months;
mod simulation;
mod speed;
mod timer;

use simulation::Simulation;

fn main() {
    let mut sim = Simulation::new();

    sim.run();
}
