// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;
use word_grid_game_core::PlayMode;

use crate::flow::RoundScreen;
use crate::match_plugin::{ActiveMatch, GameSet};

const MELODY: [f32; 8] = [261.63, 329.63, 392.0, 329.63, 293.66, 349.23, 440.0, 349.23];

#[derive(Component)]
struct RoundMusic;

#[derive(Resource, Default)]
struct MusicSequencer {
    since_beat: f32,
    beat_index: usize,
}

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicSequencer>()
            .add_systems(OnEnter(RoundScreen::Playing), reset_music)
            .add_systems(OnExit(RoundScreen::Playing), stop_music)
            .add_systems(OnEnter(RoundScreen::Review), play_finish_sting)
            .add_systems(
                Update,
                play_timed_music
                    .in_set(GameSet::Presentation)
                    .run_if(in_state(RoundScreen::Playing)),
            );
    }
}

fn reset_music(mut sequencer: ResMut<MusicSequencer>) {
    sequencer.since_beat = f32::MAX;
    sequencer.beat_index = 0;
}

fn play_timed_music(
    time: Res<Time>,
    game: Res<ActiveMatch>,
    mut sequencer: ResMut<MusicSequencer>,
    mut pitches: ResMut<Assets<Pitch>>,
    mut commands: Commands,
) {
    let Some(round) = game.0.current_round() else {
        return;
    };
    let PlayMode::TimedRounds { duration } = game.0.rules().play_mode() else {
        return;
    };
    let Some(remaining) = round.remaining() else {
        return;
    };
    let progress = 1.0 - remaining.as_secs_f32() / duration.as_secs_f32();
    let interval = beat_interval(progress);
    sequencer.since_beat += time.delta_secs();
    if sequencer.since_beat < interval {
        return;
    }
    sequencer.since_beat = 0.0;
    let octave = if progress > 0.85 { 1.5 } else { 1.0 };
    let frequency = MELODY[sequencer.beat_index % MELODY.len()] * octave;
    sequencer.beat_index += 1;
    commands.spawn((
        AudioPlayer(pitches.add(Pitch::new(frequency, Duration::from_millis(130)))),
        PlaybackSettings {
            volume: Volume::Linear(0.055),
            ..PlaybackSettings::DESPAWN
        },
        RoundMusic,
    ));
}

fn stop_music(mut commands: Commands, playing: Query<Entity, With<RoundMusic>>) {
    for entity in &playing {
        commands.entity(entity).despawn();
    }
}

fn play_finish_sting(mut pitches: ResMut<Assets<Pitch>>, mut commands: Commands) {
    for frequency in [523.25, 659.25, 783.99] {
        commands.spawn((
            AudioPlayer(pitches.add(Pitch::new(frequency, Duration::from_millis(600)))),
            PlaybackSettings {
                volume: Volume::Linear(0.065),
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}

fn beat_interval(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    0.78 - 0.58 * progress.powi(3)
}

#[cfg(test)]
mod tests {
    use super::beat_interval;

    #[test]
    fn music_accelerates_toward_round_completion() {
        assert!(beat_interval(0.0) > beat_interval(0.75));
        assert!(beat_interval(0.75) > beat_interval(1.0));
        assert_eq!(beat_interval(-1.0), beat_interval(0.0));
        assert_eq!(beat_interval(2.0), beat_interval(1.0));
    }
}
