use crate::states::AppState;
use bevy::app::AppExit;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::utils::tracing;
use bevy::window::PrimaryWindow;
use std::time::Duration;

const CAMERA_SNAP_MULT: f32 = 8.0;
const BASE_MOVE_SPEED: f32 = 128.0;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<SimTag>()
			.register_type::<TopText>()
			.register_type::<Player>()
			.register_type::<Players>()
			.add_systems(OnEnter(AppState::Simulation), spawn_sim)
			.add_systems(OnExit(AppState::Simulation), despawn_sim)
			.add_systems(FixedUpdate, handle_sim_input.run_if(in_state(AppState::Simulation)))
			.add_systems(FixedUpdate, update_sim.run_if(in_state(AppState::Simulation)))
			.add_systems(Update, display_ui.run_if(in_state(AppState::Simulation)))
			.add_systems(PostUpdate, gizmo_render.run_if(in_state(AppState::Simulation)));
	}
}

#[derive(Component, Reflect)]
pub struct SimTag;

#[derive(Component, Reflect)]
pub struct TopText;

#[derive(Component, Reflect)]
pub struct Player {
	pub name: String,
	pub color: Color,
}

#[derive(Resource, Debug, Reflect)]
pub struct Players {
	pub players: Vec<Option<Entity>>,
}

#[tracing::instrument(skip(commands, exit))]
fn spawn_sim(
	mut commands: Commands,
	mut time: ResMut<FixedTime>,
	window_query: Query<&Window, With<PrimaryWindow>>,
	mut exit: EventWriter<AppExit>,
) {
	time.period = Duration::from_secs_f64(1.0 / 120.0);
	// This is for pixel-perfect tilemap rendering
	commands.insert_resource(Msaa::Off);
	let Ok(window) = window_query.get_single() else {
		error!("Failed to get primary window");
		exit.send(AppExit);
		return;
	};
	commands.spawn((
		SimTag,
		Camera2dBundle {
			transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
			camera_2d: Camera2d {
				clear_color: ClearColorConfig::Custom(Color::BLACK),
			},
			..Default::default()
		},
	));
	commands.spawn((
		SimTag,
		TopText,
		TextBundle::from_sections([
			TextSection::new("-TEMP0-", TextStyle::default()),
			TextSection::new("-TEMP1-", TextStyle::default()),
		])
		.with_background_color(Color::DARK_GRAY)
		.with_style(Style {
			display: Display::Flex,
			position_type: PositionType::Absolute,
			left: Val::Px(0.0),
			top: Val::Px(0.0),
			width: Val::Px(312.0),
			height: Val::Px(48.0),
			..Style::DEFAULT
		}),
	));
	let player = commands
		.spawn((
			SimTag,
			Player {
				name: "Tester".to_string(),
				color: Color::GREEN,
			},
			SpriteBundle {
				sprite: Sprite::default(),
				transform: Transform::default(),
				global_transform: GlobalTransform::default(),
				texture: Handle::default(),
				visibility: Visibility::default(),
				computed_visibility: ComputedVisibility::default(),
			},
		))
		.id();
	commands.insert_resource(Players {
		players: vec![Some(player)],
	});
}

#[tracing::instrument(skip(commands))]
fn despawn_sim(mut commands: Commands, query: Query<Entity, With<SimTag>>) {
	commands.remove_resource::<Msaa>();
	for entity in query.iter() {
		commands.entity(entity).despawn_recursive();
	}
}

#[tracing::instrument(skip(exit))]
fn handle_sim_input(
	time: Res<FixedTime>,
	mut exit: EventWriter<AppExit>,
	keyboard_input: Res<Input<KeyCode>>,
	players: Res<Players>,
	mut player_query: Query<&mut Transform, With<Player>>,
) {
	if keyboard_input.just_pressed(KeyCode::Escape) {
		exit.send(AppExit);
	}
	{
		let mut player_translation_delta = Vec3::ZERO;
		if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
			player_translation_delta.y += 1.0;
		}
		if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
			player_translation_delta.x -= 1.0;
		}
		if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
			player_translation_delta.y -= 1.0;
		}
		if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
			player_translation_delta.x += 1.0;
		}
		let player_translation_delta =
			player_translation_delta.normalize_or_zero() * (time.period.as_secs_f32() * BASE_MOVE_SPEED);
		if let Some(player_id) = players.players.get(0).copied().flatten() {
			if let Ok(mut player_transform) = player_query.get_mut(player_id) {
				let player_translation = &mut player_transform.translation;
				*player_translation += player_translation_delta;
				player_translation.z = 0.0;
			}
		}
	};
}

#[tracing::instrument(skip())]
fn update_sim(
	time: Res<FixedTime>,
	mut camera: Query<&mut Transform, (With<SimTag>, With<Camera>)>,
	players: Query<&GlobalTransform, With<Player>>,
) {
	let camera_destination = {
		let (summed, count) = players
			.iter()
			.map(GlobalTransform::translation)
			.fold((Vec3::ZERO, 0f32), |(sum, count), position| {
				(sum + position, count + 1.0)
			});
		summed / count
	};
	let camera_translation = &mut camera.single_mut().translation;
	*camera_translation = camera_translation.lerp(
		camera_destination,
		time.period.as_secs_f32().min(1.0) * CAMERA_SNAP_MULT,
	);
}

#[tracing::instrument(skip())]
fn display_ui(
	time: Res<Time>,
	fixed_time: Res<FixedTime>,
	mut top_text_query: Query<&mut Text, With<TopText>>,
	player_query: Query<(&Player, &GlobalTransform)>,
) {
	let Ok(mut text) = top_text_query.get_single_mut() else {
		error!("Failed to get top text");
		return;
	};
	text.sections[0].value = format!("TickTime: {:?} Frametime: {:?}\n", fixed_time.period, time.delta());
	let player = player_query.single();
	text.sections[1].value = format!("{}: {:?}", &player.0.name, player.1.translation());
}

#[tracing::instrument(skip(gizmo))]
fn gizmo_render(mut gizmo: Gizmos, query: Query<(&Player, &GlobalTransform)>) {
	for (player, transform) in query.iter() {
		gizmo.rect_2d(
			transform.translation_vec3a().truncate(),
			0.0,
			Vec2::new(32.0, 32.0),
			player.color,
		);
	}
}
