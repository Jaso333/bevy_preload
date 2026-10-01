use bevy::{
    app::MainScheduleOrder, asset::LoadedUntypedAsset, ecs::schedule::ScheduleLabel, prelude::*,
};

pub mod prelude {
    pub use crate::*;
}

/// The startup schedule to run after everything has preloaded.
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone)]
pub struct PreloadedStartup;

/// Happens before [`First`] so that [`PreloadedStartup`] is effectively the same as [`Startup`] in terms of organisation.
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone)]
struct PreloadCheck;

/// Contains all systems in this module.
#[derive(SystemSet, Hash, Clone, Debug, PartialEq, Eq)]
pub struct PreloadSystems;

/// The overall state of the preload functionality.
#[derive(Resource, Default, Debug)]
struct PreloadState {
    /// The paths of the assets to load.
    paths: Vec<&'static str>,
    /// The assets still loading.
    loading: Vec<Handle<LoadedUntypedAsset>>,
    /// The assets that have loaded.
    loaded: Vec<UntypedHandle>,
    /// Flags when the app has started, so it doesn't occur more than once.
    started: bool,
}

/// Adds preload functionality to the app.
pub struct PreloadPlugin;

impl Plugin for PreloadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PreloadState>();

        app.init_schedule(PreloadCheck);
        app.init_schedule(PreloadedStartup);

        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_before(First, PreloadCheck);

        app.add_systems(Update, update_assets.in_set(PreloadSystems));

        app.add_systems(PreloadCheck, check_completion);
    }
}

/// Adds preloading options to app building.
pub trait PreloadAppExt {
    /// Adds a list of asset paths to preload.
    fn preload_assets(&mut self, paths: impl Into<Vec<&'static str>>) -> &mut Self;
}

impl PreloadAppExt for App {
    fn preload_assets(&mut self, paths: impl Into<Vec<&'static str>>) -> &mut Self {
        self.world_mut()
            .resource_mut::<PreloadState>()
            .paths
            .append(&mut paths.into());

        self
    }
}

/// Updates the asset loading part of the preload.
fn update_assets(
    mut state: ResMut<PreloadState>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<LoadedUntypedAsset>>,
) {
    if state.started {
        return;
    }

    let mut new_loading = state
        .paths
        .drain(..)
        .map(|path| asset_server.load_builder().load_untyped(path))
        .collect();

    state.loading.append(&mut new_loading);

    let mut new_loaded = Vec::new();
    state.loading.retain(|handle| {
        if let Some(asset) = assets.get(handle.id()) {
            new_loaded.push(asset.handle.clone());
            return false;
        }
        true
    });

    state.loaded.append(&mut new_loaded);
}

/// Checks the state for completion.
fn check_completion(mut state: ResMut<PreloadState>, mut commands: Commands) {
    if state.started {
        return;
    }

    if state.paths.is_empty() && state.loading.is_empty() {
        state.started = true;
        info!("preloaded {} assets", state.loaded.len());
        commands.run_schedule(PreloadedStartup);
    }
}
