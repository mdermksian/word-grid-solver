// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

mod app;
mod board;
mod content;
mod flow;
mod hud;
mod match_plugin;

use bevy::prelude::*;

pub use app::{build_app, run};
pub use flow::{RoundScreen, Screen};
pub use match_plugin::{GameSet, MatchNotice, PlayerIntent};

use board::BoardPlugin;
use content::ContentPlugin;
use flow::FlowPlugin;
use hud::HudPlugin;
use match_plugin::MatchPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
enum StartupSet {
    Content,
    Match,
    Presentation,
}

pub struct WordGridGamePlugin;

impl Plugin for WordGridGamePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Startup,
            (
                StartupSet::Content,
                StartupSet::Match,
                StartupSet::Presentation,
            )
                .chain(),
        )
        .configure_sets(
            Update,
            (GameSet::Input, GameSet::Domain, GameSet::Presentation).chain(),
        )
        .add_plugins((
            ContentPlugin,
            FlowPlugin,
            MatchPlugin,
            BoardPlugin,
            HudPlugin,
        ));
    }
}
