use bevy::prelude::*;

use crate::{level::CurrentLevel, GameState};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(level_input.in_set(OnUpdate(GameState::Playing)));
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
