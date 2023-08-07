use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod assets;
mod camera;
mod particles;
mod player;
mod score;
mod world;

const PLAYER_GROUP: Group = Group::GROUP_1;
const WORLD_GROUP: Group = Group::GROUP_2;
const PARTICLE_GROUP: Group = Group::GROUP_3;

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum GameState {
    #[default]
    Loading,
    Playing,
    ScoreScreen,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(
                    bevy_embedded_assets::EmbeddedAssetPlugin,
                )
                .set(ImagePlugin::default_nearest()),
        );

        app.add_state::<GameState>();
        app.add_systems(Update, bevy::window::close_on_esc);

        #[cfg(feature = "debug")]
        {
            app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default());
            app.add_plugins(bevy_rapier2d::prelude::RapierDebugRenderPlugin::default());

            app.add_plugins(bevy_debug_text_overlay::OverlayPlugin::default());
            app.insert_resource(GizmoConfig {
                depth_bias: -1.0,
                ..default()
            });
        }
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.0));

        app.add_plugins((
            assets::AssetPlugin,
            player::PlayerPlugin,
            camera::CameraPlugin,
            world::WorldPlugin,
            particles::ParticlesPlugin,
            score::ScorePlugin,
        ));

        app.add_systems(OnExit(GameState::Playing), kill_entities_in_playing);
    }
}

#[derive(Component, Default)]
struct InPlayingOnly;

fn kill_entities_in_playing(mut commands: Commands, query: Query<Entity, With<InPlayingOnly>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
