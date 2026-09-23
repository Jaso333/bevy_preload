use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_preload::prelude::*;

const RED_IMAGE_PATH: &str = "red.png";
const GREEN_IMAGE_PATH: &str = "green.png";
const BLUE_IMAGE_PATH: &str = "blue.png";

#[derive(Component, Clone, Default)]
struct Square;

fn game() -> impl SceneList {
    bsn_list![
        Camera2d,

        Square
        Sprite {
            image: RED_IMAGE_PATH
        }
        Transform::from_xyz(-256.0, 0.0, 0.0),

        Square
        Sprite {
            image: GREEN_IMAGE_PATH
        },

        Square
        Sprite {
            image: BLUE_IMAGE_PATH
        }
        Transform::from_xyz(256.0, 0.0, 0.0),

        Sprite {
            color: Color::WHITE,
            custom_size: { Some(Vec2::new(1920.0, 1080.0)) }
        }
        Transform::from_xyz(0.0, 0.0, -1.0),
    ]
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PreloadPlugin)
        .preload_assets(vec![RED_IMAGE_PATH, GREEN_IMAGE_PATH, BLUE_IMAGE_PATH])
        .add_systems(PreloadedStartup, startup)
        .add_systems(Update, rotate_squares)
        .run();
}

fn startup(mut commands: Commands) {
    info!("loaded assets!");

    commands.spawn_scene_list(game());
}

fn rotate_squares(mut square_query: Query<&mut Transform, With<Square>>, time: Res<Time>) {
    for mut transform in square_query.iter_mut() {
        transform.rotate_z(time.delta_secs() * PI);
    }
}
