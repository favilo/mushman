use bevy::{app::AppExit, prelude::*};

use crate::{GameState, level::Coord};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DeathEvent(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Sound {
    PlayerDie,
    HitWall,
    Explosion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundEvent {
    pub sound: Sound,
}

pub struct MovementEvent {
    pub pos: Coord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelEvent(pub usize);

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DeathEvent>()
            .add_event::<SoundEvent>()
            .add_event::<MovementEvent>()
            .add_event::<LevelEvent>()
            .add_system(player_death.in_set(OnUpdate(GameState::Playing)));
    }
}

fn player_death(
    mut _commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut events: EventReader<DeathEvent>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for event in events.iter() {
        // commands.spawn_bundle(TextBundle {
        //     text: Text::with_section(
        //         event.0.clone(),
        //         TextStyle {
        //             font: Asset::<Font>::load("fonts/FiraSans-Bold.ttf"),
        //             font_size: 40.0,
        //             color: Color::WHITE,
        //         },
        //         TextAlignment::default(),
        //     ),
        //     ..Default::default()
        // });
        // TODO: Handle GameOver state properly
        log::info!("{}", event.0);
        next_state.set(GameState::GameOver);
        app_exit_events.send(AppExit);
    }
}
