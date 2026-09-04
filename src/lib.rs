use bevy::{
    asset::LoadedUntypedAsset,
    prelude::*,
    render::{MainWorld, RenderApp, render_resource::PipelineCache},
};

pub mod prelude {
    pub use crate::{
        PreloadAssetsComplete, PreloadAssetsIncomplete, PreloadAssetsManifest, PreloadPlugin,
        PreloadState, PreloadSystems,
    };
}

/// The set that contains all preload systems within the assigned schedule.
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PreloadSystems;

/// Contains the list of assets to preload. Spawn this to initiate preloading of the identified assets.
#[derive(Component, Default, Clone)]
#[require(
    PreloadAssetsIncomplete,
    PreloadAssetsComplete,
    PreloadPipelinesTimer,
    PreloadPipelinesCount,
    PreloadState
)]
pub struct PreloadAssetsManifest(pub Vec<&'static str>);

/// The assets that are currently preloading. This is required by the manifest.
#[derive(Component, Default)]
pub struct PreloadAssetsIncomplete(pub Vec<Handle<LoadedUntypedAsset>>);

/// The assets that have preloaded. This ensures that the assets are always pinned and therefore never dropped.
/// This is required by the manifest.
#[derive(Component, Default, Clone)]
pub struct PreloadAssetsComplete(pub Vec<UntypedHandle>);

/// The timer used to be sure that pipelines have loaded.
/// This is due to the fact that the preload count can drop to zero temporarily before pushing more to the queue.
#[derive(Component, Clone)]
pub struct PreloadPipelinesTimer(pub Timer);

impl Default for PreloadPipelinesTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Once))
    }
}

/// Records the number of pipelines currently being loaded.
#[derive(Component, Default, Clone)]
pub struct PreloadPipelinesCount(pub usize);

/// The state being tracked from when the manifest is propagated.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum PreloadState {
    #[default]
    Loading,
    Loaded,
}

/// Adds preload functionality to the app.
pub struct PreloadPlugin;

impl Plugin for PreloadPlugin {
    fn build(&self, app: &mut App) {
        // Use "First" schedule as the app is likely to start mass-spawning entities when the preload completes.
        // This also supports the nature of the plugin: *pre*-loading before anything happens.
        // Apps would typically consider this point to be the "true" startup point, like the "Startup" schedule.
        app.add_systems(First, check_loaded.in_set(PreloadSystems))
            .add_systems(
                First,
                (propagate_manifests, update_handles)
                    .chain()
                    .before(check_loaded)
                    .in_set(PreloadSystems),
            )
            .add_systems(
                First,
                update_pipelines_timer
                    .before(check_loaded)
                    .in_set(PreloadSystems),
            );

        app.sub_app_mut(RenderApp).add_systems(
            ExtractSchedule,
            update_pipelines_count.in_set(PreloadSystems),
        );
    }
}

/// Ingests the latest manifest when it has changed, kicking off the loading of each asset.
fn propagate_manifests(
    mut manifest_query: Query<
        (
            &PreloadAssetsManifest,
            &mut PreloadAssetsIncomplete,
            &mut PreloadAssetsComplete,
            &mut PreloadState,
        ),
        Changed<PreloadAssetsManifest>,
    >,
    asset_server: Res<AssetServer>,
) {
    for (manifest, mut incomplete, mut complete, mut state) in manifest_query.iter_mut() {
        // a new manifest resets everything
        incomplete.0 = manifest
            .0
            .iter()
            .map(|path| asset_server.load_builder().load_untyped(*path))
            .collect();

        // clear the previous preloaded assets as they no longer represent the manifest
        complete.0.clear();

        // reset the state
        *state = PreloadState::Loading;
    }
}

/// Updates the loading and loaded handles based on if they are loaded.
fn update_handles(
    mut handle_query: Query<(&mut PreloadAssetsIncomplete, &mut PreloadAssetsComplete)>,
    loaded_untyped_assets: Res<Assets<LoadedUntypedAsset>>,
) {
    for (mut incomplete, mut complete) in handle_query.iter_mut() {
        let mut new_loaded = Vec::new();
        incomplete.0.retain(|handle| {
            if let Some(asset) = loaded_untyped_assets.get(handle.id()) {
                new_loaded.push(asset.handle.clone());
                return false;
            }
            true
        });

        complete.0.append(&mut new_loaded);
    }
}

/// Checks for the condition for when the assets are loaded.
fn check_loaded(
    mut state_query: Query<(
        &mut PreloadState,
        &PreloadAssetsIncomplete,
        &PreloadPipelinesTimer,
    )>,
) {
    for (mut state, incomplete, timer) in state_query.iter_mut() {
        if state.as_ref() == &PreloadState::Loaded
            || !incomplete.0.is_empty()
            || !timer.0.is_finished()
        {
            continue;
        }

        *state = PreloadState::Loaded;
    }
}

/// During extraction, log the number of pipelines waiting in the cache.
fn update_pipelines_count(mut main_world: ResMut<MainWorld>, cache: Res<PipelineCache>) {
    let mut query = main_world.query::<&mut PreloadPipelinesCount>();

    for mut count in query.iter_mut(&mut main_world) {
        count.0 = cache.waiting_pipelines().count();
    }
}

/// Updates the timer for confirming pipelines have finished loading.
fn update_pipelines_timer(
    mut preload_query: Query<(&PreloadPipelinesCount, &mut PreloadPipelinesTimer)>,
    time: Res<Time<Real>>,
) {
    for (count, mut timer) in preload_query.iter_mut() {
        if count.0 == 0 {
            if timer.0.is_finished() {
                continue;
            }

            timer.0.tick(time.delta());
        } else {
            timer.0.reset();
        }
    }
}
