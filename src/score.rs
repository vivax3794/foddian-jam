use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{assets, GameState, InPlayingOnly};

const LOOTLOCKER_API_KEY: &str = "dev_b3d730b5d65c4fdbb9a145f47a5b5902";

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunTime>();
        app.init_resource::<HasStarted>();
        app.add_systems(
            OnEnter(GameState::Playing),
            (reset_runtime, create_score_text),
        );
        app.add_systems(
            Update,
            (update_runtime, update_score_text, detect_end).run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            OnEnter(GameState::ScoreScreen),
            (create_score_screen, update_and_load_leaderboard).chain(),
        );
        app.add_systems(Update, center_item);

        app.add_systems(
            Update,
            (reset_button, update_score_text).run_if(in_state(GameState::ScoreScreen)),
        );
        app.add_systems(OnExit(GameState::ScoreScreen), kill_entities);
    }
}

#[derive(Resource, Default)]
pub struct HasStarted(pub bool);

#[derive(Resource, Default)]
struct RunTime(Duration);

fn reset_runtime(mut run_time: ResMut<RunTime>) {
    run_time.0 = Duration::default();
}

fn update_runtime(mut run_time: ResMut<RunTime>, time: Res<Time>, started: Res<HasStarted>) {
    if started.0 {
        run_time.0 += time.delta();
    }
}

#[derive(Component)]
struct ScoreTextMarker;

fn create_score_text(mut commands: Commands, misc: Res<assets::Misc>) {
    commands
        .spawn(
            TextBundle::from_section(
                "NaN",
                TextStyle {
                    font: misc.font.clone(),
                    font_size: 30.0,
                    ..default()
                },
            )
            .with_style(Style {
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                ..default()
            }),
        )
        .insert(ScoreTextMarker)
        .insert(RemoveAfterScore);
}

fn update_score_text(
    mut score_text: Query<&mut Text, With<ScoreTextMarker>>,
    run_time: Res<RunTime>,
) {
    let Ok(mut score_text) = score_text.get_single_mut() else {
        return;
    };

    let millis = run_time.0.as_millis();
    let (seconds, millis) = (millis / 1000, millis % 1000);
    let (minutes, seconds) = (seconds / 60, seconds % 60);

    score_text.sections[0].value = format!("{minutes:0>2}:{seconds:0>2}.{millis:0>3}");
}

fn detect_end(
    levels: Res<Assets<LdtkLevel>>,
    loaded_levels: Query<&Handle<LdtkLevel>>,
    selected_level: Res<LevelSelection>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(current_level) = loaded_levels.iter().find_map(|loaded_level| {
        let level = levels.get(loaded_level).unwrap();
        if selected_level.is_match(&0, &level.level) {
            Some(level)
        } else {
            None
        }
    }) else {
        return;
    };

    let &is_end = current_level.level.get_bool_field("IsEnd").unwrap();
    if is_end {
        next_state.0 = Some(GameState::ScoreScreen);
    }
}

fn create_score_screen(mut commands: Commands, misc: Res<assets::Misc>) {
    commands
        .spawn(
            TextBundle::from_section(
                "You completed the game!",
                TextStyle {
                    font: misc.font.clone(),
                    font_size: 50.0,
                    ..default()
                },
            )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(10.0),
                ..default()
            }),
        )
        .insert(Center)
        .insert(RemoveAfterScore);
}

#[derive(Component)]
struct Center;

fn center_item(mut items: Query<(&Node, &mut Style), With<Center>>, window: Query<&Window>) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let window_width = window.width();

    for (node, mut style) in items.iter_mut() {
        let item_width = node.size().x;
        let offset = window_width / 2.0 - item_width / 2.0;
        style.left = Val::Px(offset);
    }
}

#[derive(Serialize)]
struct LootlockerGuestLoginRequest {
    game_key: String,
    game_version: String,
}

#[derive(Deserialize, Debug)]
struct LootlockerGuestLoginResponse {
    session_token: String,
}

#[derive(Serialize)]
struct LootlockerLeaderboardSubmit {
    score: u128,
}

#[derive(Deserialize)]
struct LootlockerLeaderboardSubmitResponse {
    rank: usize,
}

#[derive(Deserialize)]
struct LeaderboardItem {
    score: u128,
}

#[derive(Deserialize)]
struct LeaderboardResponse {
    items: Vec<LeaderboardItem>,
}

#[cfg(feature = "web")]
fn update_and_load_leaderboard() {}

#[cfg(not(feature = "web"))]
fn update_and_load_leaderboard(
    mut commands: Commands,
    run_time: Res<RunTime>,
    misc: Res<assets::Misc>,
) {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.lootlocker.io/game/v2/session/guest")
        .json(&LootlockerGuestLoginRequest {
            game_key: LOOTLOCKER_API_KEY.into(),
            game_version: "0.0.1".into(),
        })
        .send()
        .unwrap();
    let login_response: LootlockerGuestLoginResponse = response.json().unwrap();

    let submit_response: LootlockerLeaderboardSubmitResponse = client
        .post("https://api.lootlocker.io/game/leaderboards/main/submit")
        .json(&LootlockerLeaderboardSubmit {
            score: run_time.0.as_millis(),
        })
        .header("x-session-token", login_response.session_token.clone())
        .send()
        .unwrap()
        .json()
        .unwrap();

    commands
        .spawn(
            TextBundle::from_section(
                format!("You got rank: {}", submit_response.rank),
                TextStyle {
                    font: misc.font.clone(),
                    font_size: 40.0,
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(15.0),
                ..default()
            }),
        )
        .insert(Center)
        .insert(RemoveAfterScore);

    let leaderboard_data: LeaderboardResponse = client
        .get("https://api.lootlocker.io/game/leaderboards/main/list?count=10")
        .header("x-session-token", login_response.session_token)
        .send()
        .unwrap()
        .json()
        .unwrap();

    let mut height = 20.0;
    for item in leaderboard_data.items {
        let millis = item.score;
        let (seconds, millis) = (millis / 1000, millis % 1000);
        let (minutes, seconds) = (seconds / 60, seconds % 60);

        commands
            .spawn(
                TextBundle::from_section(
                    format!("{minutes:0>2}:{seconds:0>2}.{millis:0>3}"),
                    TextStyle {
                        font: misc.font.clone(),
                        font_size: 35.0,
                        ..default()
                    },
                )
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(height),
                    ..default()
                }),
            )
            .insert(Center)
            .insert(RemoveAfterScore);
        height += 8.0;
    }
}

#[derive(Component)]
struct RemoveAfterScore;

fn kill_entities(mut commands: Commands, query: Query<Entity, With<RemoveAfterScore>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn reset_button(
    mut next_state: ResMut<NextState<GameState>>,
    input: Res<Input<KeyCode>>,
    mut level_selection: ResMut<LevelSelection>,
) {
    if input.just_pressed(KeyCode::R) {
        next_state.0 = Some(GameState::Playing);
        *level_selection = LevelSelection::Identifier(String::from("Start"));
    }
}
