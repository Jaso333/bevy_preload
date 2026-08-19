use bevy::{asset::LoadedUntypedAsset, ecs::template::TemplateContext, prelude::*};

pub mod prelude {
    pub use crate::{PreloadManifest, PreloadPlugin, PreloadState, PreloadSystems};
}

pub struct PreloadPlugin;

impl Plugin for PreloadPlugin {
    fn build(&self, app: &mut App) {
        // Use "First" schedule as the app is likely to start mass-spawning entities when the preload completes.
        // This also supports the nature of the plugin: *pre*-loading before anything happens.
        // Apps would typically consider this point to be the "true" startup point, like the "Startup" schedule.
        app.add_systems(First, preload_system.in_set(PreloadSystems));
    }
}

fn preload_system(
    mut loader_query: Query<(
        Ref<PreloadManifest>,
        &mut PreloadState,
        &mut PreloadingAssets,
        &mut PreloadedAssets,
    )>,
    loaded_untypeds: Res<Assets<LoadedUntypedAsset>>,
    asset_server: Res<AssetServer>,
) {
    for (manifest, mut state, mut loading, mut loaded) in loader_query.iter_mut() {
        if manifest.is_changed() {
            loading.0 = manifest
                .0
                .iter()
                .map(|path| asset_server.load_builder().load_untyped(*path))
                .collect();
            loaded.0.clear();

            *state = PreloadState::Loading;
        } else if loading.0.is_empty() {
            continue;
        }

        let mut new_loaded = Vec::new();
        loading.0.retain(|handle| {
            if let Some(asset) = loaded_untypeds.get(handle.id()) {
                new_loaded.push(asset.handle.clone());
                return false;
            }
            true
        });

        if new_loaded.len() > 0 && loading.0.is_empty() {
            *state = PreloadState::Loaded;
        }

        loaded.0.append(&mut new_loaded);
    }
}

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PreloadSystems;

#[derive(Component, Default, Clone)]
#[require(PreloadState, PreloadingAssets, PreloadedAssets)]
pub struct PreloadManifest(pub Vec<&'static str>);

#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum PreloadState {
    #[default]
    NotStarted,
    Loading,
    Loaded,
}

#[derive(Component, Default)]
struct PreloadingAssets(Vec<Handle<LoadedUntypedAsset>>);

impl FromTemplate for PreloadingAssets {
    type Template = PreloadingAssetsTemplate;
}

#[derive(Default)]
struct PreloadingAssetsTemplate;

impl Template for PreloadingAssetsTemplate {
    type Output = PreloadingAssets;

    fn build_template(&self, _: &mut TemplateContext) -> Result<Self::Output> {
        Ok(PreloadingAssets::default())
    }

    fn clone_template(&self) -> Self {
        Self
    }
}

#[derive(Component, Default, Clone)]
struct PreloadedAssets(Vec<UntypedHandle>);
