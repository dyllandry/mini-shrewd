use bevy::{prelude::*, render::camera::RenderTarget, transform::TransformSystem};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use chrono::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
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
        .add_startup_system(add_ground)
        .add_startup_system(add_player)
        .add_system(player_movement)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            camera_follow_player.after(TransformSystem::TransformPropagate),
        )
        .add_system(set_player_direction_from_input)
        .add_system(log_time)
        .add_system(log_positions)
        .add_system(set_clicked_clickables)
        .add_system(create_dropdown_when_inspectable_clicked)
        .add_system(ui_example);
    }
}

fn ui_example(mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
        ui.label("world");
    });
}

fn add_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn camera_follow_player(
    mut camera_transform_query: Query<&mut GlobalTransform, (With<Camera>, Without<Player>)>,
    player_transform_query: Query<&GlobalTransform, (With<Player>, Without<Camera>)>,
) {
    let player_transform = player_transform_query.single();
    let player_translation = player_transform.translation();
    let mut camera_transform = camera_transform_query.single_mut();
    let camera_translation = camera_transform.translation_mut();
    camera_translation.x = player_translation.x;
}

// Maybe rename to set_clicked_sprites
fn set_clicked_clickables(
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut clickables_query: Query<(&mut Clickable, &Transform, &Handle<Image>)>,
    mouse_buttons: Res<Input<MouseButton>>,
    assets: Res<Assets<Image>>,
) {
    // Reset all clicked components to not be clicked.
    for (mut clickable, ..) in clickables_query.iter_mut() {
        if clickable.just_clicked {
            clickable.just_clicked = false;
        }
    }

    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = camera_query.single();
    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    if let Some(screen_pos) = window.cursor_position() {
        let click_world_pos = {
            // This approach comes from https://bevy-cheatbook.github.io/cookbook/cursor2world.html
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
            world_pos.truncate()
        };

        let clicked_clickables_query_element =
            clickables_query
                .iter_mut()
                .find(|(_clickable, transform, image_handle)| {
                    let image_asset = assets.get(image_handle);
                    return match image_asset {
                        None => false,
                        Some(image_asset) => {
                            // One problem with this is that an image asset's size is the size of the
                            // image file's width and height. So if I draw a small object in a 32x32
                            // file, the small object's size will be 32x32. A way around this is to crop
                            // the image to the sprite's bounds before I save the file.
                            // A longer solution is when an asset loads to sample it's pixels to find its
                            // bounds, then set that on a new component like SpritePixelBounds, then
                            // iterate through those in this system instead of image assets.
                            let image_size = image_asset.size();
                            println!("clickable image size: {:?}", image_size);

                            // If using sprites with different anchors, query for the sprite component
                            // and account for anchor while calculating sprite world bounds.
                            let sprite_world_bounds_min_x: f32 =
                                { transform.translation.x - (image_size.x / 2.0) };
                            let sprite_world_bounds_max_x: f32 =
                                { transform.translation.x + (image_size.x / 2.0) };
                            let sprite_world_bounds_min_y: f32 =
                                { transform.translation.y - (image_size.y / 2.0) };
                            let sprite_world_bounds_max_y: f32 =
                                { transform.translation.y + (image_size.y / 2.0) };

                            return click_world_pos.x <= sprite_world_bounds_max_x
                                && click_world_pos.x >= sprite_world_bounds_min_x
                                && click_world_pos.y <= sprite_world_bounds_max_y
                                && click_world_pos.y >= sprite_world_bounds_min_y;
                        }
                    };
                });

        if let Some((mut clickable, ..)) = clicked_clickables_query_element {
            // I want to figure out what to do now.
            // Can add property to Clickable component "just_clicked".
            // Systems that care about certain things being clicked can query for that component
            // then see if Clickable.just_clicked is true & do their logic.
            // This system set_clicked_images at the start will have to iterate through all
            // clickables and set just_clicked to false.
            clickable.just_clicked = true;
            println!("clicked on image!");
        }
    }
}

fn create_dropdown_when_inspectable_clicked(query: Query<(&Clickable, &Inspectable, &Transform)>) {
    for (clickable, _inspectable, _transform) in query.iter() {
        if clickable.just_clicked {
            println!("Just clicked an inspectable thing!");
        }
    }
}

fn add_trees(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("vicky's tree.png"),
        transform: Transform::from_xyz(100.0, 0.0, SpriteLayers::Trees as i32 as f32),
        ..default()
    });
}

fn add_ground(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tile_size = 32.0;
    let num_tiles = 10;
    let mut tile_y = 0.0 - ((tile_size * num_tiles as f32) / 2.0);
    for _i in 0..num_tiles {
        let mut tile_x = 0.0 - ((tile_size * num_tiles as f32) / 2.0);
        for _j in 0..num_tiles {
            commands.spawn_bundle(SpriteBundle {
                texture: asset_server.load("ground.png"),
                transform: Transform::from_xyz(tile_x, tile_y, SpriteLayers::Ground as i32 as f32),
                ..default()
            });
            tile_x += tile_size;
        }
        tile_y += tile_size;
    }
}

fn add_player(mut commands: Commands, asset_server: Res<AssetServer>, _assets: Res<Assets<Image>>) {
    let player_image_handle: Handle<Image> = asset_server.load("finley.png");
    let player_translation = Vec3::new(0.0, 0.0, SpriteLayers::Player as i32 as f32);
    commands
        .spawn_bundle(SpriteBundle {
            texture: player_image_handle,
            transform: Transform::from_translation(player_translation),
            ..default()
        })
        .insert(Player {})
        .insert(Direction { vec: Vec3::ZERO })
        .insert(Clickable::new())
        .insert(Inspectable {});
}

fn player_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Sprite, &Direction), With<Player>>,
) {
    if let Some((mut player_transform, mut sprite, direction)) = query.iter_mut().next() {
        player_transform.translation += direction.vec * 50.0 * time.delta_seconds();
        if direction.vec.x > 0.0 {
            sprite.flip_x = false;
        } else if direction.vec.x < 0.0 {
            sprite.flip_x = true;
        }
    }
}

fn set_player_direction_from_input(
    mut query: Query<&mut Direction, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Some(mut player_direction) = query.iter_mut().next() {
        let mut new_direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::W) {
            new_direction.y += 1.0;
        }

        if keyboard_input.pressed(KeyCode::S) {
            new_direction.y += -1.0;
        }

        if keyboard_input.pressed(KeyCode::D) {
            new_direction.x += 1.0;
        }

        if keyboard_input.pressed(KeyCode::A) {
            new_direction.x += -1.0;
        }

        player_direction.vec = new_direction;
    }
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

#[derive(Component)]
struct Player {}

#[derive(Component)]
struct Direction {
    vec: Vec3,
}

enum SpriteLayers {
    Ground,
    Player,
    Trees,
}

#[derive(Component, Default)]
struct Clickable {
    just_clicked: bool,
}

impl Clickable {
    fn new() -> Self {
        self::default()
    }
}

#[derive(Component)]
struct Inspectable {}
