use bevy::prelude::*;

use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{assets, particles::SpawnParticles, score, GameState, PLAYER_GROUP, WORLD_GROUP};

const FIRE_STRENGTH: f32 = 40.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlayerBundle>("PlayerStart");

        app.add_systems(
            Update,
            (
                setup_player,
                handle_player_input,
                show_uses_on_left,
                show_uses_on_right,
                refresh_uses,
                change_level_we_are_in,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Default)]
pub struct PlayerMarker;

#[derive(Bundle)]
struct PlayerPhysics {
    collider: Collider,
    group: CollisionGroups,
    bounce: Restitution,
    body: RigidBody,
    impulse: ExternalImpulse,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            collider: Collider::ball(12.0 / 2.0),
            group: CollisionGroups::new(PLAYER_GROUP, WORLD_GROUP),
            bounce: Restitution {
                coefficient: 1.0,
                combine_rule: CoefficientCombineRule::Average,
            },
            body: RigidBody::Dynamic,
            impulse: ExternalImpulse::default(),
        }
    }
}

#[derive(Component)]
struct GunsUsed {
    left: bool,
    right: bool,
    minimal_time: Timer,
}

impl Default for GunsUsed {
    fn default() -> Self {
        Self {
            left: true,
            right: true,
            minimal_time: Timer::from_seconds(0.100, TimerMode::Once),
        }
    }
}

#[derive(Bundle, LdtkEntity)]
struct PlayerBundle {
    marker: PlayerMarker,
    guns: GunsUsed,

    trans: TransformBundle,
    vis: VisibilityBundle,

    #[worldly]
    worldly: Worldly,

    physics: PlayerPhysics,
}

#[derive(Component)]
struct RingMarker;

#[derive(Component)]
struct LeftMarker;

#[derive(Component)]
struct RightMarker;

fn setup_player(
    mut commands: Commands,
    players: Query<Entity, Added<PlayerMarker>>,
    misc: Res<assets::Misc>,
) {
    for player in players.iter() {
        commands.entity(player).with_children(|children| {
            children.spawn(SpriteSheetBundle {
                texture_atlas: misc.player.clone(),
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..default()
                },
                ..default()
            });
            children
                .spawn(SpriteSheetBundle {
                    texture_atlas: misc.player.clone(),
                    sprite: TextureAtlasSprite {
                        index: 1,
                        ..default()
                    },
                    ..default()
                })
                .insert(RingMarker);
            children
                .spawn(SpriteSheetBundle {
                    texture_atlas: misc.player.clone(),
                    sprite: TextureAtlasSprite {
                        index: 2,
                        ..default()
                    },
                    ..default()
                })
                .insert(LeftMarker);
            children
                .spawn(SpriteSheetBundle {
                    texture_atlas: misc.player.clone(),
                    sprite: TextureAtlasSprite {
                        index: 3,
                        ..default()
                    },
                    ..default()
                })
                .insert(RightMarker);
        });
    }
}

fn handle_player_input(
    mut commands: Commands,
    mut players: Query<(&mut ExternalImpulse, &Transform, &mut GunsUsed), With<PlayerMarker>>,
    input: Res<Input<MouseButton>>,
    mut particle_events: EventWriter<SpawnParticles>,
    mut started: ResMut<score::HasStarted>,
    audio: Res<assets::Sound>,
) {
    let Ok((mut impulse, transform, mut guns)) = players.get_single_mut() else {
        return;
    };

    let shoot_direction;

    let raw_pulse = if input.just_pressed(MouseButton::Left) && guns.left {
        guns.left = false;
        shoot_direction = Vec2::new(-1.0, 0.0);
        Vec2::new(FIRE_STRENGTH, 0.0)
    } else if input.just_pressed(MouseButton::Right) && guns.right {
        guns.right = false;
        shoot_direction = Vec2::new(1.0, 0.0);
        Vec2::new(-FIRE_STRENGTH, 0.0)
    } else {
        return;
    };

    let rotation = transform.rotation;
    let rotated_pulse = rotation.mul_vec3(raw_pulse.extend(0.0)).truncate();
    impulse.impulse = rotated_pulse;

    let particle_local_pos = shoot_direction * 8.0;
    let particle_local_direction = shoot_direction;

    let particle_world_pos = rotation.mul_vec3(particle_local_pos.extend(0.0)).truncate()
        + transform.translation.truncate();
    let particle_world_direction = rotation
        .mul_vec3(particle_local_direction.extend(0.0))
        .truncate();

    particle_events.send(SpawnParticles {
        location: particle_world_pos,
        direction: particle_world_direction,
    });

    commands.spawn(AudioBundle {
        source: audio.shoot.clone(),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            ..default()
        },
    });

    guns.minimal_time.reset();
    started.0 = true;
}

fn show_uses_on_left(
    mut left: Query<&mut TextureAtlasSprite, With<LeftMarker>>,
    player: Query<&GunsUsed, With<PlayerMarker>>,
) {
    let Ok(guns) = player.get_single() else {
        return;
    };
    let Ok(mut left) = left.get_single_mut() else {
        return;
    };

    left.color = if guns.left {
        Color::WHITE
    } else {
        Color::DARK_GRAY
    };
}
fn show_uses_on_right(
    mut right: Query<&mut TextureAtlasSprite, With<RightMarker>>,
    player: Query<&GunsUsed, With<PlayerMarker>>,
) {
    let Ok(guns) = player.get_single() else {
        return;
    };
    let Ok(mut right) = right.get_single_mut() else {
        return;
    };

    right.color = if guns.right {
        Color::WHITE
    } else {
        Color::DARK_GRAY
    };
}

fn refresh_uses(
    mut player: Query<(&mut GunsUsed, &Transform), With<PlayerMarker>>,
    context: Res<RapierContext>,
    time: Res<Time>,
) {
    let Ok((mut guns, trans)) = player.get_single_mut() else {
        return;
    };

    if !guns.minimal_time.tick(time.delta()).finished() {
        return;
    }

    let trans = trans.translation;
    let shape_pos = trans.truncate();
    let shape_dir = Vec2::new(0.0, -1.0);
    let max_toi = 0.5;
    let filter = QueryFilter::exclude_dynamic();
    let shape = Collider::ball(6.0);

    if context
        .cast_shape(shape_pos, 0.0, shape_dir, &shape, max_toi, filter)
        .is_some()
    {
        guns.left = true;
        guns.right = true;
    }
}

fn change_level_we_are_in(
    player_query: Query<&Transform, With<PlayerMarker>>,
    mut level_selection: ResMut<LevelSelection>,
    levels: Res<Assets<LdtkLevel>>,
    loaded_levels: Query<(&Transform, &Handle<LdtkLevel>), Without<PlayerMarker>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for (level_transform, level_handel) in loaded_levels.iter() {
        let level = levels.get(level_handel).unwrap();

        let level_corner_1 = level_transform.translation.truncate();
        let level_size = Vec2::new(level.level.px_wid as f32, level.level.px_hei as f32);
        let level_corner_2 = level_corner_1 + level_size;
        let rect = Rect::from_corners(level_corner_1, level_corner_2);

        if rect.contains(player_transform.translation.truncate()) {
            let new = LevelSelection::Iid(level.level.iid.clone());
            if new != *level_selection {
                *level_selection = new;
                // commands.entity(level_entity).insert(Respawn);
            }
            break;
        }
    }
}
