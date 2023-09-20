#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)] // Bevy uses this pattern a lot for systems
#![allow(clippy::module_name_repetitions)] // Bevy uses this pattern a lot for Plugins

pub mod states;

use bevy::prelude::*;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(states::StatePlugin)
		.add_systems(Startup, switch_to_sim_now)
		.run();
}

fn switch_to_sim_now(mut app_state: ResMut<NextState<states::AppState>>) {
	app_state.set(states::AppState::Simulation);
}
