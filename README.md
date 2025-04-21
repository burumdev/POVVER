# POVVER

A simulator of simplified manufacturing economics and energy distribution between industries.
It follows a multithreaded, actor style, concurrent timeline and event management. The program has a native desktop UI.

## Description

POVVER has a simplified supply and demand economy model which can be described as
a caricature of real world economics.

The main focus is on energy production and distribution networks.
Renewable energy trading and networking between simulated enterprises (factories)
and its comparison to a more traditional, fossil fuel based energy conglomerate is the
main point of inspiration.
In addition to factories, a traditional "Povver Plant" that uses fossil fuels is also modelled.

The program can be thought of as an experiment in "Fearless Concurrency" promise of Rust programming language.
It has a distributed, multithreaded, event driven structure and tries to enforce principles of least priviledge and
single source of truth by exposing entity states as read-only to themselves.

As a result, a central processing entity called "The Hub" takes the role of modifying entity states, not themselves.
An ideal approach to turn this setup to a completely trustless network would be to use a blockchain with smart contracts like Solana.
That might be a goal for later versions.

POVVER is currently early alpha software. Scaffolding of the simulation and primary dynamics is in place.
UI is also in a pretty mature state. But there are areas that need improvement:

* Simulation needs tuning to make it yield more realistic and convincing results.
* Renewable energy sale between enterprises is not implemented yet
* Wind turbines are not implemented
* Demand from the economy and production from factories is not properly balanced or timed (tuning)
* Bugs to be ironed-out

POVVER uses the declarative Slint library for desktop UI and written in Rust for backend and Slint for frontend.

## License

POVVER is licensed under the terms of GNU General Public License version 3. Later version of this license will not apply until further consideration.
See the LICENSE file in project root and main.rs file in project src directory for details.
For third party licenses of UI elements and fonts see Image Assets and Fonts sections.

## Attributions
### Image Asset Licenses

POVVER uses images from the SVG library EmojiOne version 2, which is no longer supported or distributed.
The license is Creative Commons Attribution 4.0 International License. NPM page of emojione is https://www.npmjs.com/package/emojione and homepage is https://www.emojione.com/

### Font Licenses

The program uses Digital-7 font from http://www.styleseven.com/ 
for LCD-like timer display, which is free to use for a freeware software like POVVER.
Thanks for this awesome font!

Monaspice Nerd Font is used for other regular text. Which can be found here: https://www.nerdfonts.com/font-downloads

### Slint UI Library

Slint is used for a native desktop UI. https://slint.dev/

## Running

Though POVVER is also published on Crates.io (main Rust code repository)
cloning it from GitHub is highly recommended.

This requires an installation of Rust build system.
See instructions on how to install rustup here: https://www.rust-lang.org/tools/install

After cloning enter this command to run the simulator:

```
cargo run -r
```

### OS Platforms

POVVER is in early alpha stage and is not tested on Windows and MacOS systems.

## Authors

Barış Ürüm a.k.a burumdev (barisurum.works@gmail.com)

## Version History

* 0.1.0
    * Initial Release (alpha)

## Contributing

Anyone is welcome to contribute to the project with bug fixes or feature implementations.
This project is licensed under GNU General Public License version 3 only.
Further iterations of the GNU General Public License won't apply unless license is updated.
All contributions will be licensed under the same GPL v3 license.