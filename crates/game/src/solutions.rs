// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::collections::HashSet;

use bevy::prelude::*;
use word_grid_game_core::ScoringTable;
use word_grid_solver::FoundWord;

#[derive(Component)]
pub(crate) struct SolutionWord {
    pub path: Vec<usize>,
    pub was_submitted: bool,
}

pub(crate) fn spawn_solution_words(
    list: &mut ChildSpawnerCommands,
    words: &[FoundWord],
    scoring: &ScoringTable,
    submitted: &HashSet<&str>,
) {
    for solved in words {
        let was_submitted = submitted.contains(solved.word.as_str());
        list.spawn((
            Button,
            SolutionWord {
                path: solved.path.clone(),
                was_submitted,
            },
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(36.0),
                padding: UiRect::horizontal(Val::Px(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(if was_submitted {
                Color::srgb(0.12, 0.32, 0.18)
            } else {
                Color::srgb(0.22, 0.15, 0.1)
            }),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(if was_submitted {
                    format!("[x] {}", solved.word.to_uppercase())
                } else {
                    solved.word.to_uppercase()
                }),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor::WHITE,
            ));
            button.spawn((
                Text::new(scoring.score_word(&solved.word).to_string()),
                TextFont {
                    font_size: FontSize::Px(17.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.82, 0.35)),
            ));
        });
    }
}
