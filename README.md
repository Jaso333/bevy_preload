# Design

It is often desirable in game development to load all assets up-front before presenting anything to the user. This library introduces a simple implementation for the following flow:

1. Game start
1. Start loading assets
1. Wait for assets to load
1. Load the first level

This is realised with a simple `Preload` component and a `PreloadComplete` event.

# Usage

Add the crate to your project:

```toml
bevy_preload = "0.1.0"
```

Add the plugin:

```rust
app.add_plugins(PreloadPlugin);
```

Spawn an entity or even a scene:

```rust
commands.spawn_scene(bsn! {
    bsn! {
        Preload::from_paths(["game_sprite.png", "game_sound.wav", "some_asset.ron"])
        on(|_: On<PreloadComplete>, mut commands: Commands| commands.spawn_scene_list(game()))
    }
});
```

In the above example, the event is used to start the game once the preload is complete.

# License

bevy_starling is free and open source. All code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([LICENSE-APACHE](/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.