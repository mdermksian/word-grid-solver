// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
#[states(scoped_entities)]
pub enum Screen {
    Loading,
    Menu,
    #[default]
    Match,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(Screen = Screen::Match)]
#[states(scoped_entities)]
pub enum RoundScreen {
    Rolling,
    #[default]
    Playing,
    Review,
}

pub struct FlowPlugin;

impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Screen>().add_sub_state::<RoundScreen>();
    }
}
