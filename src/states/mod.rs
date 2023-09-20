pub mod simulation;

use bevy::prelude::*;

pub struct StatePlugin;

impl Plugin for StatePlugin {
	fn build(&self, app: &mut App) {
		app.add_state::<AppState>().add_plugins(simulation::SimulationPlugin);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum AppState {
	#[default]
	Loading,
	Menu,
	Simulation,
}
