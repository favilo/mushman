use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioPlugin;
use iyes_progress::ProgressPlugin;

mod events;
mod image;
mod input;
mod level;

use events::EventPlugin;
use image::TexturePlugin;
use input::InputPlugin;
use level::LevelPlugin;

#[derive(Debug, Default, Clone, Copy, States, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    Playing,
    Paused,
    GameOver,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_state::<GameState>()
        // TODO: Change this to Menu, and create a menu
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Playing),
        )
        .add_plugin(AudioPlugin)
        .add_plugin(EventPlugin)
        .add_plugin(
            ProgressPlugin::new(GameState::Loading)
                .continue_to(GameState::Menu)
                .track_assets(),
        )
        .add_plugin(InputPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(TexturePlugin)
        .add_startup_system(setup)
        // .add_system(move_camera.system())
        .run();
}

#[derive(Debug, Copy, Clone, Component)]
pub struct LevelRoot;

#[derive(Debug, Copy, Clone, Component)]
pub struct Player;

fn setup(mut commands: Commands) {
    // TODO: Figure out a better camera position
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(7.0 * 32.0, -8.0 * 32.0, 10.0),
        ..Default::default()
    });

    commands.spawn(LevelRoot);
}
