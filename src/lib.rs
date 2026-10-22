use bevy_app::prelude::*;
use bevy_asset::{LoadedUntypedAsset, prelude::*};
use bevy_ecs::{prelude::*, template::TemplateContext};
use bevy_log::prelude::*;

/// Provides general-use "preload" functionality.
///
/// Use the [`Preload`] component to specify a list of asset paths to load and be notified on completion.
///
/// The [`PreloadComplete`] event is triggered when the preload is complete.
pub struct PreloadPlugin;

impl Plugin for PreloadPlugin {
    fn build(&self, app: &mut App) {
        // preloading starts at the end of the frame, giving the opportunity to run logic in update for filling the paths
        app.add_systems(PostUpdate, begin_loading.in_set(PreloadSystems));

        // completion happens at the start of the frame, allowing event handlers to spawn things before `Update`
        app.add_systems(PreUpdate, end_loading.in_set(PreloadSystems));
    }
}

/// The set used for systems in this library.
#[derive(SystemSet, PartialEq, Eq, Hash, Clone, Debug)]
pub struct PreloadSystems;

/// Triggered on a [`Preload`] entity when loading is complete.
#[derive(EntityEvent)]
pub struct PreloadComplete {
    /// The entity containing the assets that have loaded.
    pub entity: Entity,
}

/// Specifies the assets to load and contains the loaded assets.
///
/// This component ensures that the loaded assets are not unloaded by holding reference to them.
///
/// This component acts as a "hopper" of sorts, where paths are fed in and eventually the equivalent handles are loaded.
/// Because of this, it is up to the user to control and limit the times that paths are added,
/// as the [`PreloadComplete`] event is triggered every time the [`Preload::loading`] field is emptied.
///
/// A typical use for a simple game would be to throw all possible asset paths into this component on [`Startup`] and then begin the game upon completion.
#[derive(Component, Default)]
pub struct Preload {
    /// The paths to consume in the next execution of the system.
    /// These are converted to handles in the [`Preload::loading`] field.
    pub paths: Vec<String>,
    /// The handles currently that are currently loading.
    /// These are consumed and converted to handles in the [`Preload::loaded`] when loaded.
    pub loading: Vec<Handle<LoadedUntypedAsset>>,
    /// The loaded handles. These pin the assets, ensuring that are not unloaded for as long as this component exists.
    pub loaded: Vec<UntypedHandle>,
}

impl Preload {
    /// Convenience constructor for generating the component from an iterator of paths.
    pub fn from_paths<I>(paths: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        Self {
            paths: paths.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

/// This is necessary due to the [`Preload::loading`] field in this component, unfortunately.
impl FromTemplate for Preload {
    type Template = Self;
}

impl Template for Preload {
    type Output = Self;

    fn build_template(&self, _: &mut TemplateContext) -> Result<Self::Output> {
        Ok(Self::clone_template(self))
    }

    fn clone_template(&self) -> Self {
        Self {
            paths: self.paths.clone(),
            loading: self.loading.clone(),
            loaded: self.loaded.clone(),
        }
    }
}

/// Begins loading any paths currently present in the [`Preload`] components.
fn begin_loading(mut preload_query: Query<&mut Preload>, asset_server: Res<AssetServer>) {
    for mut preload in preload_query.iter_mut() {
        let mut loading = preload
            .paths
            .drain(..)
            .map(|path| asset_server.load_builder().load_untyped(path))
            .collect::<Vec<_>>();

        preload.loading.append(&mut loading);
    }
}

/// Ends the loading of any handles currently in the loading state.
fn end_loading(
    mut preload_query: Query<(Entity, &mut Preload)>,
    assets: Res<Assets<LoadedUntypedAsset>>,
    mut commands: Commands,
) {
    for (entity, mut preload) in preload_query.iter_mut() {
        if preload.loading.is_empty() {
            continue;
        }

        // collect the loaded handles
        let mut loaded = Vec::new();
        preload.loading.retain(|handle| {
            if let Some(asset) = assets.get(handle.id()) {
                loaded.push(asset.handle.clone());
                return false;
            }

            true
        });

        preload.loaded.append(&mut loaded);

        // some loading completed and there is none left, so we can trigger the event
        if preload.loading.is_empty() {
            info!(
                "preloading complete, {} assets available",
                preload.loaded.len()
            );
            commands.trigger(PreloadComplete { entity });
        }
    }
}
