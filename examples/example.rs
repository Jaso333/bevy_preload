use bevy::prelude::*;
use bevy_preload::prelude::*;

const RED_IMAGE_PATH: &str = "red.png";
const GREEN_IMAGE_PATH: &str = "green.png";
const BLUE_IMAGE_PATH: &str = "blue.png";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PreloadPlugin)
        .add_systems(Startup, game_assets_scene.spawn())
        .add_systems(First, assets_loaded_system.after(PreloadSystems))
        .run();
}

fn assets_loaded_system(
    preload_query: Query<&PreloadState, Changed<PreloadState>>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
) {
    if preload_query.is_empty() {
        return;
    }

    let state = preload_query.single().unwrap();
    if state != &PreloadState::Loaded {
        return;
    }

    // won't fail because the assets are known to be loaded a this point
    let red_image = images.get(asset_server.load(RED_IMAGE_PATH).id()).unwrap();
    let green_image = images
        .get(asset_server.load(GREEN_IMAGE_PATH).id())
        .unwrap();
    let blue_image = images.get(asset_server.load(BLUE_IMAGE_PATH).id()).unwrap();

    let red_color = red_image.get_color_at(0, 0).unwrap();
    let green_color = green_image.get_color_at(0, 0).unwrap();
    let blue_color = blue_image.get_color_at(0, 0).unwrap();

    println!("red is {red_color:?}");
    println!("green is {green_color:?}");
    println!("blue is {blue_color:?}");
}

fn game_assets_scene() -> impl Scene {
    bsn! {
        PreloadManifest(vec![RED_IMAGE_PATH, GREEN_IMAGE_PATH, BLUE_IMAGE_PATH])
    }
}
