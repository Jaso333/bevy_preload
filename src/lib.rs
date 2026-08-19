use bevy::{asset::LoadedUntypedAsset, ecs::template::TemplateContext, prelude::*};

pub struct PreloadPlugin;

impl Plugin for PreloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, preload_system.in_set(PreloadSystems));
    }
}

fn preload_system(
    mut loader_query: Query<(
        Entity,
        Ref<AssetManifest>,
        &mut LoadingAssets,
        &mut LoadedAssets,
    )>,
    loaded_untypeds: Res<Assets<LoadedUntypedAsset>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (entity, manifest, mut loading, mut loaded) in loader_query.iter_mut() {
        if manifest.is_changed() {
            loading.0 = manifest
                .0
                .iter()
                .map(|path| asset_server.load_builder().load_untyped(*path))
                .collect();
            loaded.0.clear();
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
            commands.trigger(AssetsLoaded(entity));
        }

        loaded.0.append(&mut new_loaded);
    }
}

pub fn preload_scene() -> impl Scene {
    bsn! {
        AssetManifest
        LoadingAssets
        LoadedAssets
    }
}

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PreloadSystems;

#[derive(EntityEvent)]
pub struct AssetsLoaded(pub Entity);

#[derive(Component, Default, Clone)]
pub struct AssetManifest(pub Vec<&'static str>);

#[derive(Component, Default)]
pub struct LoadingAssets(Vec<Handle<LoadedUntypedAsset>>);

impl FromTemplate for LoadingAssets {
    type Template = LoadingAssetsTemplate;
}

#[derive(Default)]
pub struct LoadingAssetsTemplate;

impl Template for LoadingAssetsTemplate {
    type Output = LoadingAssets;

    fn build_template(&self, _: &mut TemplateContext) -> Result<Self::Output> {
        Ok(LoadingAssets::default())
    }

    fn clone_template(&self) -> Self {
        Self
    }
}

#[derive(Component, Default, Clone)]
pub struct LoadedAssets(Vec<UntypedHandle>);
