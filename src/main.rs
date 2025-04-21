/*
POVVER
Copyright (C) 2025 Barış Ürüm <barisurum.works@gmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License version 3
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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
