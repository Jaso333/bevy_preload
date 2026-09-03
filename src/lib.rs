use bevy::{asset::LoadedUntypedAsset, prelude::*};

pub mod prelude {
    pub use crate::{
        PreloadManifest, PreloadPlugin, PreloadState, PreloadSystems, PreloadedAssetHandles,
        PreloadingAssetHandles,
    };
}

/// The set that contains all preload systems within the assigned schedule.
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PreloadSystems;

/// Contains the list of assets to preload. Spawn this to initiate preloading of the identified assets.
#[derive(Component, Default, Clone)]
#[require(PreloadingAssetHandles, PreloadedAssetHandles, PreloadState)]
pub struct PreloadManifest(pub Vec<&'static str>);

/// The assets that are currently preloading. This is required by the manifest.
#[derive(Component, Default)]
pub struct PreloadingAssetHandles(pub Vec<Handle<LoadedUntypedAsset>>);

/// The assets that have preloaded. This ensures that the assets are always pinned and therefore never dropped.
/// This is required by the manifest.
#[derive(Component, Default, Clone)]
pub struct PreloadedAssetHandles(pub Vec<UntypedHandle>);

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
        app.add_systems(
            First,
            (propagate_manifests, update_handles, check_loaded)
                .chain()
                .in_set(PreloadSystems),
        );
    }
}

/// Ingests the latest manifest when it has changed, kicking off the loading of each asset.
fn propagate_manifests(
    mut manifest_query: Query<
        (
            &PreloadManifest,
            &mut PreloadingAssetHandles,
            &mut PreloadedAssetHandles,
            &mut PreloadState,
        ),
        Changed<PreloadManifest>,
    >,
    asset_server: Res<AssetServer>,
) {
    for (manifest, mut preloading, mut preloaded, mut state) in manifest_query.iter_mut() {
        // a new manifest resets everything
        preloading.0 = manifest
            .0
            .iter()
            .map(|path| asset_server.load_builder().load_untyped(*path))
            .collect();

        // clear the previous preloaded assets as they no longer represent the manifest
        preloaded.0.clear();

        // reset the state
        *state = PreloadState::Loading;
    }
}

/// Updates the loading and loaded handles based on if they are loaded.
fn update_handles(
    mut handle_query: Query<(&mut PreloadingAssetHandles, &mut PreloadedAssetHandles)>,
    loaded_untyped_assets: Res<Assets<LoadedUntypedAsset>>,
) {
    for (mut loading, mut loaded) in handle_query.iter_mut() {
        let mut new_loaded = Vec::new();
        loading.0.retain(|handle| {
            if let Some(asset) = loaded_untyped_assets.get(handle.id()) {
                new_loaded.push(asset.handle.clone());
                return false;
            }
            true
        });

        loaded.0.append(&mut new_loaded);
    }
}

/// Checks for the condition for when the assets are loaded.
fn check_loaded(mut state_query: Query<(&mut PreloadState, &PreloadingAssetHandles)>) {
    for (mut state, loading) in state_query.iter_mut() {
        if state.as_ref() == &PreloadState::Loaded || !loading.0.is_empty() {
            continue;
        }

        *state = PreloadState::Loaded;
    }
}
