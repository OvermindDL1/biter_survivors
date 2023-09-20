use crate::states::AppState;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(AppState::Simulation), spawn_sim)
			.add_systems(OnExit(AppState::Simulation), despawn_sim)
			.add_systems(PreUpdate, handle_sim_input.run_if(in_state(AppState::Simulation)))
			.add_systems(Update, update_sim.run_if(in_state(AppState::Simulation)));
	}
}

fn spawn_sim(
	mut commands: Commands,
	window_query: Query<&Window, With<PrimaryWindow>>,
	mut exit: EventWriter<AppExit>,
) {
	// This is for pixel-perfect tilemap rendering
	commands.insert_resource(Msaa::Off);
	let Ok(window) = window_query.get_single() else {
		error!("Failed to get primary window");
		exit.send(AppExit);
		return;
	};
	commands.spawn(Camera2dBundle {
		transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
		..Default::default()
	});
}

fn despawn_sim(mut commands: Commands) {
	commands.remove_resource::<Msaa>();
}

fn handle_sim_input(mut exit: EventWriter<AppExit>, keyboard_input: Res<Input<KeyCode>>) {
	if keyboard_input.just_pressed(KeyCode::Escape) {
		exit.send(AppExit);
	}
}

fn update_sim() {}
