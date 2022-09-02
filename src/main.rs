use bevy::prelude::*;
use chrono::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MiniShrewd)
        .run();
}

pub struct MiniShrewd;

impl Plugin for MiniShrewd {
    fn build(&self, app: &mut App) {
        app.insert_resource(LogTimeTimer {
            timer: Timer::from_seconds(1.0, true),
        })
        .add_startup_system(add_camera)
        .add_startup_system(add_trees)
        .add_startup_system(add_player)
        .add_system(log_time)
        .add_system(log_positions);
    }
}

fn add_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn add_trees(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("vicky's tree.png"),
        transform: Transform::from_xyz(100.0, 0.0, 0.0),
        ..default()
    });
}

fn add_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("finley.png"),
        ..default()
    });
}

fn log_positions(query: Query<&Position>) {
    for position in query.iter() {
        println!("entity at position ({}, {})", position.x, position.y);
    }
}

fn log_time(time: Res<Time>, mut timer: ResMut<LogTimeTimer>) {
    if timer.timer.tick(time.delta()).just_finished() {
        match Duration::from_std(time.time_since_startup()) {
            Ok(run_duration) => println!(
                "time is {}:{}:{}",
                run_duration.num_hours(),
                run_duration.num_minutes() % 60,
                run_duration.num_seconds() % 60
            ),
            Err(_) => (),
        }
    }
}

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

struct LogTimeTimer {
    timer: Timer,
}
