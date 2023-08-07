use bevy::{prelude::*, window::CursorGrabMode};

use crate::{player, GameState};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_camera);
        app.add_systems(
            Update,
            (snap_camera_to_player, hide_cursor).run_if(in_state(GameState::Playing)),
        );
    }
}

fn create_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.2,
            far: 1500.0,
            ..default()
        },
        ..default()
    });
}

fn snap_camera_to_player(
    mut camera: Query<&mut Transform, With<Camera>>,
    player: Query<&GlobalTransform, With<player::PlayerMarker>>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };
    let Ok(player) = player.get_single() else {
        return;
    };

    let player_pos = player.translation().truncate();
    let camera_pos = camera.translation.truncate();

    let change = player_pos - camera_pos;
    let speed = change.length().powi(2) / 10_00.0 + 1.0;
    let change = Vec2::ZERO.lerp(change, (time.delta_seconds() * speed).min(1.0));

    camera.translation.x += change.x;
    camera.translation.y += change.y;
}

fn hide_cursor(mut window: Query<&mut Window>) {
    let Ok(mut window) = window.get_single_mut() else {
        return;
    };

    window.cursor.visible = cfg!(feature = "debug");
    window.cursor.grab_mode = CursorGrabMode::Locked;
}
