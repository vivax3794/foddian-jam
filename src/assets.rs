use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::LdtkAsset;

use crate::GameState;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Playing),
        );
        app.add_collection_to_loading_state::<_, Misc>(GameState::Loading);
        app.add_collection_to_loading_state::<_, Sound>(GameState::Loading);
    }
}
#[derive(AssetCollection, Resource)]
pub struct Misc {
    #[asset(path = "world.ldtk")]
    pub world: Handle<LdtkAsset>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 4, rows = 1))]
    #[asset(path = "player.png")]
    pub player: Handle<TextureAtlas>,
    #[asset(path = "spark.png")]
    pub spark: Handle<Image>,
    #[asset(path = "font.ttf")]
    pub font: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct Sound {
    #[asset(path = "shoot.wav")]
    pub shoot: Handle<AudioSource>,
}
