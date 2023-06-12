use bevy::prelude::*;

use crate::{
    events::{LevelEvent, MovementEvent},
    level::{load_level, CurrentLevel, LevelMap},
    GameState,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(level_input.in_set(OnUpdate(GameState::Playing)))
            // .add_system(next_level.in_set(OnUpdate(GameState::Playing)))
            .add_system(
                player_input
                    .in_set(OnUpdate(GameState::Playing))
                    .after(load_level),
            );
    }
}

fn level_input(input: Res<Input<KeyCode>>, mut current_level: ResMut<CurrentLevel>) {
    if input.just_pressed(KeyCode::J) {
        **current_level += 1;
    } else if input.just_pressed(KeyCode::K) {
        if **current_level >= 1 {
            **current_level -= 1;
        }
    }
}

// fn next_level(mut level_events: EventReader<LevelEvent>, mut current_level: ResMut<CurrentLevel>) {
//     for LevelEvent(level) in level_events.iter() {
//         **current_level = *level;
//     }
// }

fn player_input(
    input: Res<Input<KeyCode>>,
    level_map: Res<LevelMap>,
    mut movement_events: EventWriter<MovementEvent>,
) {
    let (mut dx, mut dy): (isize, isize) = (0, 0);
    if input.just_pressed(KeyCode::W) {
        dy -= 1;
    }
    if input.just_pressed(KeyCode::D) {
        dx += 1;
    }
    if input.just_pressed(KeyCode::A) {
        dx -= 1;
    }
    if input.just_pressed(KeyCode::S) {
        dy += 1;
    }

    let level_map = level_map.as_ref();
    let (height, width) = &level_map.map.dim();
    let mut pos = level_map.player_pos;
    pos.1 = (pos.1 as isize + dx).clamp(0, (*width as isize - 1).max(0)) as usize;
    pos.0 = (pos.0 as isize + dy).clamp(0, (*height as isize - 1).max(0)) as usize;
    if pos != level_map.player_pos {
        movement_events.send(MovementEvent { pos });
    }
}
