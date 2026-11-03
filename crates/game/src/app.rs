// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::path::PathBuf;

use bevy::prelude::*;

use crate::WordGridGamePlugin;

pub fn build_app() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Word Grid".to_string(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: asset_path(),
                ..default()
            }),
    )
    .add_plugins(MeshPickingPlugin)
    .add_plugins(WordGridGamePlugin);
    app
}

pub fn run() {
    build_app().run();
}

fn asset_path() -> String {
    let bundled_assets = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|directory| directory.join("assets")));

    bundled_assets
        .filter(|path| path.is_dir())
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets"))
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::asset_path;

    #[test]
    fn development_asset_path_points_at_workspace_assets() {
        assert!(std::path::Path::new(&asset_path()).is_dir());
    }
}
