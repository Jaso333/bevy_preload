use bevy::{
    app::MainScheduleOrder,
    asset::LoadedUntypedAsset,
    ecs::schedule::ScheduleLabel,
    prelude::*,
    render::{MainWorld, RenderApp, render_resource::PipelineCache},
};

pub mod prelude {
    pub use crate::*;
}

/// The maximum number of frames to wait for pipelines.
const MAX_FRAME_COUNT: usize = 5;

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
    /// The number of frames passed whilst the waiting pipeline count is zero.
    frame_count: usize,
    /// Flags when the app has started, so it doesn't occur more than once.
    started: bool,
}

/// Adds preload functionality to the app.
pub struct PreloadPlugin;

impl Plugin for PreloadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PreloadState>();

        app.init_schedule(PreloadCheck);

        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_before(First, PreloadCheck);

        app.sub_app_mut(RenderApp).add_systems(
            ExtractSchedule,
            check_waiting_pipelines.in_set(PreloadSystems),
        );

        app.add_systems(Update, update_assets.in_set(PreloadSystems));

        app.add_systems(PreloadCheck, check_completion);
    }
}

/// Adds preloading options to app building.
pub trait PreloadAppExt {
    /// Adds a list of asset paths to preload.
    fn preload_assets(&mut self, paths: Vec<&'static str>) -> &mut Self;
}

impl PreloadAppExt for App {
    fn preload_assets(&mut self, mut paths: Vec<&'static str>) -> &mut Self {
        self.world_mut()
            .resource_mut::<PreloadState>()
            .paths
            .append(&mut paths);

        self
    }
}

/// During extract, checks the waiting pipeline count.
fn check_waiting_pipelines(mut main_world: ResMut<MainWorld>, cache: Res<PipelineCache>) {
    let mut state = main_world.resource_mut::<PreloadState>();

    if state.started {
        return;
    }

    if cache.waiting_pipelines().count() == 0 {
        if state.frame_count < MAX_FRAME_COUNT {
            state.frame_count += 1;
        }
    } else {
        state.frame_count = 0;
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

    if state.frame_count >= MAX_FRAME_COUNT && state.paths.is_empty() && state.loading.is_empty() {
        state.started = true;
        commands.run_schedule(PreloadedStartup);
    }
}
