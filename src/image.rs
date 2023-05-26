use bevy::prelude::*;
use bevy_asset_loader::prelude::{AssetCollection, LoadingStateAppExt};

use crate::GameState;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, TextureData>(GameState::Loading);
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
