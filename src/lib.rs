use bevy::{asset::LoadedUntypedAsset, prelude::*};

pub mod prelude {
    pub use crate::{
        PreloadManifest, PreloadPlugin, PreloadSystems, PreloadedAssetHandles,
        PreloadingAssetHandles,
    };
}

/// The set that contains all preload systems within the assigned schedule.
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PreloadSystems;

/// Contains the list of assets to preload. Spawn this to initiate preloading of the identified assets.
#[derive(Component, Default, Clone)]
pub struct PreloadManifest(pub Vec<&'static str>);

/// The assets that are currently preloading. This is inserted when the manifest is consumed.
#[derive(Component, Default)]
pub struct PreloadingAssetHandles(pub Vec<Handle<LoadedUntypedAsset>>);

/// The assets that have preloaded. This ensures that the assets are always pinned and therefore never dropped.
/// This is inserted when the manifest is consumed.
#[derive(Component, Default, Clone)]
pub struct PreloadedAssetHandles(pub Vec<UntypedHandle>);

/// Adds preload functionality to the app.
pub struct PreloadPlugin;

impl Plugin for PreloadPlugin {
    fn build(&self, app: &mut App) {
        // Use "First" schedule as the app is likely to start mass-spawning entities when the preload completes.
        // This also supports the nature of the plugin: *pre*-loading before anything happens.
        // Apps would typically consider this point to be the "true" startup point, like the "Startup" schedule.
        app.add_systems(
            First,
            (consume_manifests, update_handles, check_completion)
                .chain()
                .in_set(PreloadSystems),
        );
    }
}

/// Ingests the latest manifest, kicking of the loading of each asset.
/// This will remove the manifest from the entity.
fn consume_manifests(
    manifest_query: Query<(Entity, &PreloadManifest), Changed<PreloadManifest>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (entity, manifest) in manifest_query.iter() {
        let preloading_asset_handles = PreloadingAssetHandles(
            manifest
                .0
                .iter()
                .map(|path| asset_server.load_builder().load_untyped(*path))
                .collect(),
        );

        commands
            .entity(entity)
            .insert((preloading_asset_handles, PreloadedAssetHandles::default()))
            .remove::<PreloadManifest>();
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

/// Removes the [`PreloadingAssetHandles`] when they have all loaded.
/// The combination of the [`PreloadedAssetHandles`] and the lack of a [`PreloadingAssetHandles`] implies completion that users can hook into.
fn check_completion(
    mut handle_query: Query<(Entity, &PreloadingAssetHandles, Ref<PreloadedAssetHandles>)>,
    mut commands: Commands,
) {
    for (entity, loading, loaded) in handle_query.iter_mut() {
        if loaded.is_changed() && loading.0.is_empty() {
            commands.entity(entity).remove::<PreloadingAssetHandles>();
        }
    }
}
