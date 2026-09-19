use bevy::prelude::*;
use bevy_preload::prelude::*;

const RED_IMAGE_PATH: &str = "red.png";
const GREEN_IMAGE_PATH: &str = "green.png";
const BLUE_IMAGE_PATH: &str = "blue.png";

fn asset_preload() -> impl Scene {
    bsn! {
        AssetPreloadManifest {
            paths: vec![RED_IMAGE_PATH, GREEN_IMAGE_PATH, BLUE_IMAGE_PATH]
        }
        on(preload_finished)
    }
}

fn game() -> impl SceneList {
    bsn_list![
        Camera2d,

        Sprite {
            image: RED_IMAGE_PATH
        }
        Transform::from_xyz(-256.0, 0.0, 0.0),

        Sprite {
            image: GREEN_IMAGE_PATH
        },

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
        .add_systems(Startup, asset_preload.spawn())
        .run();
}

fn preload_finished(_: On<PreloadFinished>, mut commands: Commands) {
    info!("loaded assets!");

    commands.spawn_scene_list(game());
}
