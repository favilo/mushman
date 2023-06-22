use bevy::{prelude::*, utils::Duration};
use bevy_asset_loader::prelude::{AssetCollection, LoadingStateAppExt};

use crate::GameState;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, TextureData>(GameState::Loading)
            .add_system(explosion_animation.in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Debug, Clone, Default, Resource, AssetCollection)]
pub struct TextureData {
    #[asset(texture_atlas(
        tile_size_x = 32.,
        tile_size_y = 32.,
        columns = 6,
        rows = 7,
        padding_x = 1.,
        padding_y = 1.
    ))]
    #[asset(path = "images/sprites.png")]
    pub atlas: Handle<TextureAtlas>,
}

#[derive(Debug, Clone, Component)]
pub struct Explosion {
    pub timer: Timer,
    pub texture_index: usize,
}

impl Default for Explosion {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(30), TimerMode::Repeating),
            texture_index: 0,
        }
    }
}

pub(crate) const EXPLOSION_INDICES: [usize; 4] = [24, 25, 30, 31];

fn explosion_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Explosion, &mut TextureAtlasSprite)>,
) {
    for (entity, mut explosion, mut sprite) in query.iter_mut() {
        if explosion.timer.tick(time.delta()).just_finished() {
            explosion.texture_index += 1;
            if explosion.texture_index >= 4 {
                commands.entity(entity).despawn_recursive();
                continue;
            }
            sprite.index = EXPLOSION_INDICES[explosion.texture_index];
        }
    }
}
