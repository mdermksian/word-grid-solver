// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use word_grid_game_core::{CubeSet, Dictionary, GameRules};

#[derive(Asset, TypePath, Debug)]
pub(crate) struct DictionaryAsset(pub Dictionary);

#[derive(Default, TypePath)]
struct DictionaryAssetLoader;

impl AssetLoader for DictionaryAssetLoader {
    type Asset = DictionaryAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Dictionary::from_bytes(&bytes).map(DictionaryAsset)
    }

    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
}

#[derive(Resource)]
pub(crate) struct ContentCatalog {
    pub rules: GameRules,
    pub dictionary: Handle<DictionaryAsset>,
    pub cube_scene: Handle<WorldAsset>,
}

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<DictionaryAsset>()
            .init_asset_loader::<DictionaryAssetLoader>()
            .add_systems(Startup, load_content);
    }
}

fn load_content(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ContentCatalog {
        rules: GameRules::normal(CubeSet::standard_new()).expect("built-in Normal rules are valid"),
        dictionary: asset_server.load("dictionaries/twl06.txt"),
        cube_scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/game_cube.glb")),
    });
}

#[cfg(test)]
mod tests {
    use word_grid_game_core::Dictionary;

    #[test]
    fn bundled_dictionary_is_platform_neutral_text() {
        let dictionary =
            Dictionary::from_bytes(include_bytes!("../../../assets/dictionaries/twl06.txt"))
                .unwrap();
        assert!(dictionary.is_word_valid("word"));
    }
}
