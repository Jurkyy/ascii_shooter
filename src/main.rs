use bevy::prelude::*;

mod player;
mod level;
mod rendering;
mod combat;
mod enemies;

/// Game states
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ASCII Boomer Shooter".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_plugins((
            player::PlayerPlugin,
            level::LevelPlugin,
            rendering::AsciiRenderPlugin,
        ))
        .add_systems(Update, (
            handle_game_state_input,
            rendering::update_ascii_resolution,
        ))
        .run();
}

fn handle_game_state_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    match current_state.get() {
        GameState::Menu => {
            if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
                next_state.set(GameState::Playing);
            }
        }
        GameState::Playing => {
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Paused);
            }
        }
        GameState::Paused => {
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Playing);
            }
            if keyboard.just_pressed(KeyCode::KeyQ) {
                next_state.set(GameState::Menu);
            }
        }
    }
}
