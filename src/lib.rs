use bevy::{
    asset::LoadedUntypedAsset,
    prelude::*,
    render::{MainWorld, RenderApp, render_resource::PipelineCache},
};

pub mod prelude {
    pub use crate::*;
}

/// The number of frames to wait for the pipeline count to settle on zero.
const PIPELINE_COUNT_SETTLE_FRAME_COUNT: u32 = 5;

/// The set that contains the systems in this module.
#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct PreloadSystems;

/// The data representing the loading state of pipelines.
#[derive(Resource, Default)]
struct PipelinePreloadData {
    /// The number of frames that the waiting count has been 0 for.
    frame_count: u32,
}

/// A manifest of assets to preload.
#[derive(Component, Default, Clone)]
pub struct AssetPreloadManifest {
    /// The paths of the assets to load.
    pub paths: Vec<&'static str>,
}

/// The internal state of the asset preloader.
#[derive(Component, Default, Clone)]
struct AssetPreloadHandles {
    /// The assets still loading.
    loading: Vec<Handle<LoadedUntypedAsset>>,
    /// The assets that have loaded.
    loaded: Vec<UntypedHandle>,
}

/// The general internal state for any preload operation.
/// This effectively forms the join between asset and pipeline loads.
#[derive(Component, Default, Clone)]
struct PreloadJoin {
    /// Flags if the entire preload is complete.
    is_finished: bool,
}

/// The event that signals when a preload has finished.
#[derive(EntityEvent)]
pub struct PreloadFinished {
    /// The entity containing the preload data.
    pub entity: Entity,
}

/// Adds the ability to preload assets and be notified of when they are complete.
/// The notification of which includes some confidence that all pipelines are also ready.
pub struct PreloadPlugin;

impl Plugin for PreloadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PipelinePreloadData>().add_systems(
            Update,
            (start_loading_assets, update_assets, finish_preload)
                .chain()
                .in_set(PreloadSystems),
        );

        app.sub_app_mut(RenderApp).add_systems(
            ExtractSchedule,
            update_pipeline_preload_data.in_set(PreloadSystems),
        );
    }
}

/// Starts loading assets when the asset preloader has changed.
fn start_loading_assets(
    manifest_query: Query<(Entity, &AssetPreloadManifest), Changed<AssetPreloadManifest>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (entity, manifest) in manifest_query.iter() {
        let loading = manifest
            .paths
            .iter()
            .map(|path| asset_server.load_builder().load_untyped(*path))
            .collect();

        commands.entity(entity).insert((
            AssetPreloadHandles {
                loading,
                ..default()
            },
            PreloadJoin::default(),
        ));
    }
}

/// Updates the assets in an asset preloader.
fn update_assets(
    mut handles_query: Query<&mut AssetPreloadHandles>,
    loaded_untyped_assets: Res<Assets<LoadedUntypedAsset>>,
) {
    for mut handles in handles_query.iter_mut() {
        let mut new_loaded = Vec::new();
        handles.loading.retain(|handle| {
            if let Some(asset) = loaded_untyped_assets.get(handle.id()) {
                new_loaded.push(asset.handle.clone());
                return false;
            }
            true
        });

        handles.loaded.append(&mut new_loaded);
    }
}

/// Gets waiting pipeline information from the render world during extract.
fn update_pipeline_preload_data(mut main_world: ResMut<MainWorld>, cache: Res<PipelineCache>) {
    let mut preloader = main_world.resource_mut::<PipelinePreloadData>();

    if cache.waiting_pipelines().count() == 0 {
        if preloader.frame_count < PIPELINE_COUNT_SETTLE_FRAME_COUNT {
            preloader.frame_count += 1;
        }
    } else {
        preloader.frame_count = 0;
    }
}

/// Checks the conditions for finishing the preload and marks it as such.
fn finish_preload(
    mut preload_query: Query<(Entity, &AssetPreloadHandles, &mut PreloadJoin)>,
    pipeline_preloader: Res<PipelinePreloadData>,
    mut commands: Commands,
) {
    for (entity, asset_preloader, mut preloader) in preload_query.iter_mut() {
        if !preloader.is_finished
            && asset_preloader.loading.len() == 0
            && pipeline_preloader.frame_count >= PIPELINE_COUNT_SETTLE_FRAME_COUNT
        {
            preloader.is_finished = true;
            commands.trigger(PreloadFinished { entity });
        }
    }
}
