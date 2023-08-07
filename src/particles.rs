use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

use crate::{assets, GameState, PARTICLE_GROUP, WORLD_GROUP};

const PARTICLE_AMOUNT: u8 = 5;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnParticles>();
        app.add_systems(
            Update,
            (spawn_particles, kill_particles).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Default)]
struct ParticleMarker;

#[derive(Component)]
struct DeathTimer(Timer);

#[derive(Event)]
pub struct SpawnParticles {
    pub location: Vec2,
    pub direction: Vec2,
}

fn spawn_particles(
    mut commands: Commands,
    mut events: EventReader<SpawnParticles>,
    misc: Res<assets::Misc>,
) {
    if cfg!(feature = "debug") {
        return;
    }

    let mut rng = thread_rng();
    for event in events.iter() {
        for _ in 0..PARTICLE_AMOUNT {
            let range = (30.0_f32).to_radians();
            let rotation = rng.gen_range(-range..range);
            let rotation_vector = Vec2::from_angle(rotation);

            let direction = event.direction.rotate(rotation_vector);
            let velocity = rng.gen_range(200.0..300.0);

            commands.spawn((
                ParticleMarker,
                SpriteBundle {
                    texture: misc.spark.clone(),
                    transform: Transform {
                        translation: event.location.extend(10.0),
                        scale: Vec3::new(0.5, 0.5, 1.0),
                        ..default()
                    },
                    ..default()
                },
                RigidBody::Dynamic,
                Collider::ball(2.0),
                CollisionGroups::new(PARTICLE_GROUP, WORLD_GROUP),
                Velocity::linear(direction * velocity),
                LockedAxes::ROTATION_LOCKED,
                Damping {
                    linear_damping: 3.0,
                    ..default()
                },
                Ccd::enabled(),
                DeathTimer(Timer::from_seconds(2.0, TimerMode::Once)),
            ));
        }
    }
}

fn kill_particles(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut DeathTimer), With<ParticleMarker>>,
    time: Res<Time>,
) {
    for (particle, mut timer) in particles.iter_mut() {
        if timer.0.tick(time.delta()).finished() {
            commands.entity(particle).despawn_recursive();
        }
    }
}
