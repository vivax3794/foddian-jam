use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::assets;
use crate::GameState;
use crate::InPlayingOnly;
use crate::WORLD_GROUP;

const DIRT: i32 = 1;
const LEFT_SLOPE: i32 = 2;
const RIGHT_SLOPE: i32 = 3;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin);
        app.insert_resource(LevelSelection::Identifier(String::from("Start")));
        app.insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            level_background: LevelBackground::Nonexistent,
            ..default()
        });
        app.add_systems(OnEnter(GameState::Playing), load_world);

        app.register_ldtk_int_cell::<DirtBundle>(DIRT);
        app.register_ldtk_int_cell::<SlopeBundle>(LEFT_SLOPE);
        app.register_ldtk_int_cell::<SlopeBundle>(RIGHT_SLOPE);
        app.register_ldtk_entity::<PlatformBundle>("Platform");

        app.add_systems(Update, (set_platform_start, move_platform));
    }
}

fn load_world(mut commands: Commands, misc: Res<assets::Misc>) {
    commands
        .spawn(LdtkWorldBundle {
            ldtk_handle: misc.world.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -100.0),
                ..default()
            },
            ..default()
        })
        .insert(InPlayingOnly);
}

fn square_collider(_: IntGridCell) -> Collider {
    Collider::cuboid(8.0, 8.0)
}

fn slope_collider(cell: IntGridCell) -> Collider {
    match cell.value {
        LEFT_SLOPE => Collider::triangle(
            Vec2::new(-8.0, -8.0),
            Vec2::new(8.0, -8.0),
            Vec2::new(-8.0, 8.0),
        ),
        RIGHT_SLOPE => Collider::triangle(
            Vec2::new(-8.0, -8.0),
            Vec2::new(8.0, -8.0),
            Vec2::new(8.0, 8.0),
        ),
        _ => unreachable!("Not a valid slope value"),
    }
}

fn world_group(_: IntGridCell) -> CollisionGroups {
    CollisionGroups::new(WORLD_GROUP, Group::ALL)
}

#[derive(Bundle, LdtkIntCell)]
struct DirtBundle {
    #[with(square_collider)]
    collider: Collider,
    #[with(world_group)]
    group: CollisionGroups,
}

#[derive(Bundle, LdtkIntCell)]
struct SlopeBundle {
    #[with(slope_collider)]
    collider: Collider,
    #[with(world_group)]
    group: CollisionGroups,
}

#[derive(Component, Default)]
struct PlatformMarker;

#[derive(Component)]
struct PlatformTarget(Vec2);

#[derive(Component)]
struct PlatformStart(Vec2);

#[derive(Component, Default)]
struct PlatformProgress(f32);

#[derive(Component, Default)]
enum PlatformDirection {
    #[default]
    Forward,
    Backwards,
}

fn platform_target(entity: &EntityInstance) -> PlatformTarget {
    let grid_pos_entity = entity.grid.as_vec2();
    let grid_pos_target = entity.get_point_field("Target").unwrap().as_vec2();

    let change = grid_pos_target - grid_pos_entity;
    let bevy_change = Vec2::new(change.x * 16.0, -change.y * 16.0);

    PlatformTarget(bevy_change)
}

fn platform_collider(_: &EntityInstance) -> Collider {
    Collider::heightfield(vec![0.5, 0.5], Vec2::new(16.0, 16.0))
}

#[derive(Bundle, LdtkEntity)]
struct PlatformBundle {
    marker: PlatformMarker,

    #[with(platform_collider)]
    collider: Collider,

    #[sprite_sheet_bundle]
    sprite: SpriteSheetBundle,
    #[with(platform_target)]
    target: PlatformTarget,
    progress: PlatformProgress,
    direction: PlatformDirection,
}

fn set_platform_start(
    mut commands: Commands,
    platforms: Query<(Entity, &Transform), Added<PlatformMarker>>,
) {
    for (platform, pos) in platforms.iter() {
        commands
            .entity(platform)
            .insert(PlatformStart(pos.translation.truncate()));
    }
}

fn move_platform(
    mut platforms: Query<
        (
            &mut Transform,
            &mut PlatformDirection,
            &mut PlatformProgress,
            &PlatformStart,
            &PlatformTarget,
        ),
        With<PlatformMarker>,
    >,
    time: Res<Time>,
) {
    for (mut transform, mut direction, mut progress, start, target) in platforms.iter_mut() {
        let target = start.0 + target.0;
        let (from, to) = match *direction {
            PlatformDirection::Forward => (start.0, target),
            PlatformDirection::Backwards => (target, start.0),
        };

        progress.0 += time.delta_seconds() / (to - from).length() * 20.0;
        progress.0 = progress.0.min(1.0);

        let new = from.lerp(to, progress.0);
        transform.translation.x = new.x;
        transform.translation.y = new.y;

        if progress.0 == 1.0 {
            *direction = match *direction {
                PlatformDirection::Forward => PlatformDirection::Backwards,
                PlatformDirection::Backwards => PlatformDirection::Forward,
            };
            progress.0 = 0.0;
        }
    }
}
