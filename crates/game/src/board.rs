// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::collections::{HashMap, HashSet};

use ab_glyph::{Font, FontArc, GlyphId, PxScale, ScaleFont, point};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::picking::prelude::*;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use word_grid_game_core::{CubeSet, MatchPhase};

use crate::content::ContentCatalog;
use crate::flow::{RoundScreen, Screen};
use crate::hud::InputDraft;
use crate::match_plugin::{ActiveMatch, GameSet, MatchNotice, PlayerIntent};

const DIE_SIZE: f32 = 1.0;
const DIE_SPACING: f32 = 1.25;
const FACE_OFFSET: f32 = DIE_SIZE / 2.0 + 0.003;
const LABEL_IMAGE_SIZE: u32 = 512;
const LABEL_FACE_SIZE: f32 = 1.64;

#[derive(Component)]
struct DieCell(usize);

#[derive(Component)]
struct BoardDie;

#[derive(Component)]
struct RollingDie {
    start: Vec3,
    target: Vec3,
    target_rotation: Quat,
    elapsed: f32,
    delay: f32,
    duration: f32,
    turns: f32,
}

#[derive(Resource, Default)]
struct RollAnimationState {
    active: bool,
    notified: bool,
}

#[derive(SystemParam)]
struct BoardAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

#[derive(Resource, Default)]
struct HighlightState(Option<WordHighlight>);

struct WordHighlight {
    path: Vec<usize>,
    timer: Timer,
}

#[derive(Debug, Clone, Message)]
pub(crate) struct BoardHighlightRequest {
    pub path: Vec<usize>,
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HighlightState>()
            .init_resource::<RollAnimationState>()
            .add_message::<BoardHighlightRequest>()
            .add_systems(OnEnter(Screen::Match), setup_environment)
            .add_systems(
                OnEnter(RoundScreen::Rolling),
                (clear_board, spawn_board).chain(),
            )
            .add_systems(
                Update,
                animate_roll
                    .in_set(GameSet::Presentation)
                    .run_if(in_state(RoundScreen::Rolling)),
            )
            .add_systems(
                Update,
                (consume_highlight_requests, tick_highlight, draw_highlight)
                    .chain()
                    .in_set(GameSet::Presentation)
                    .run_if(in_state(Screen::Match)),
            );
    }
}

fn setup_environment(
    mut commands: Commands,
    game: Res<ActiveMatch>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(1.0, 0.93, 0.82),
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });

    let table_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.31, 0.14, 0.06),
        perceptual_roughness: 0.78,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(table_material),
        DespawnOnExit(Screen::Match),
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.9, 0.75),
            illuminance: 4_500.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-3.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(Screen::Match),
    ));

    let grid_size = game.0.rules().grid_size();
    let camera_height = (grid_size as f32 * 2.125).max(8.5);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.0, camera_height, 0.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        DespawnOnExit(Screen::Match),
    ));
}

fn clear_board(mut commands: Commands, dice: Query<Entity, With<BoardDie>>) {
    for entity in &dice {
        commands.entity(entity).despawn();
    }
}

fn spawn_board(
    mut commands: Commands,
    catalog: Res<ContentCatalog>,
    game: Res<ActiveMatch>,
    mut assets: BoardAssets,
    mut roll_state: ResMut<RollAnimationState>,
    mut highlight: ResMut<HighlightState>,
) {
    roll_state.active = true;
    roll_state.notified = false;
    highlight.0 = None;

    let label_mesh = assets
        .meshes
        .add(Rectangle::new(LABEL_FACE_SIZE, LABEL_FACE_SIZE));
    let cube_set = game.0.rules().cube_set();
    let label_materials = label_materials(cube_set, &mut assets.images, &mut assets.materials);

    let grid_size = game.0.rules().grid_size();
    let positions = grid_positions(grid_size);
    let board = game
        .0
        .current_round()
        .expect("the initial match has an active round")
        .board();
    for (position_index, rolled) in board.cubes().iter().enumerate() {
        let cube = &cube_set.cubes()[rolled.cube_index()];
        let physical_faces = cube.faces().len() == face_layouts().len();
        let rotation = if physical_faces {
            orientation_for_top_face(rolled.face_index(), position_index)
        } else {
            Quat::from_rotation_y(position_index as f32 % 4.0 * std::f32::consts::FRAC_PI_2)
        };
        let displayed_labels = displayed_labels(cube.faces(), rolled.face_index());
        let target = positions[position_index] + Vec3::Y * (DIE_SIZE / 2.0);
        let column_offset = position_index % grid_size;
        let start = target
            + Vec3::new(
                (column_offset as f32 - (grid_size - 1) as f32 / 2.0) * 0.35,
                3.8 + (position_index % 3) as f32 * 0.35,
                (position_index / grid_size) as f32 * 0.22 - 0.4,
            );
        let mut die = commands.spawn((
            Name::new(format!("Die {}", position_index + 1)),
            BoardDie,
            DieCell(position_index),
            Pickable::default(),
            WorldAssetRoot(catalog.cube_scene.clone()),
            Transform::from_translation(start),
            RollingDie {
                start,
                target,
                target_rotation: rotation,
                elapsed: 0.0,
                delay: (position_index % grid_size) as f32 * 0.055,
                duration: 1.15 + (position_index % 3) as f32 * 0.08,
                turns: 2.0 + (position_index % 4) as f32 * 0.5,
            },
            DespawnOnExit(Screen::Match),
        ));

        die.observe(select_die);
        die.with_children(|parent| {
            for (face, label) in face_layouts().into_iter().zip(displayed_labels) {
                parent.spawn((
                    Mesh3d(label_mesh.clone()),
                    MeshMaterial3d(label_materials[label].clone()),
                    Transform::from_translation(face.normal * FACE_OFFSET)
                        .with_rotation(face.rotation()),
                ));
            }
        });
    }
}

fn animate_roll(
    time: Res<Time>,
    mut dice: Query<(&mut Transform, &mut RollingDie)>,
    mut state: ResMut<RollAnimationState>,
    mut intents: MessageWriter<PlayerIntent>,
) {
    if !state.active || state.notified || dice.is_empty() {
        return;
    }
    let mut finished = true;
    for (mut transform, mut rolling) in &mut dice {
        rolling.elapsed += time.delta_secs();
        let progress = ((rolling.elapsed - rolling.delay) / rolling.duration).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);
        let bounce = (progress * std::f32::consts::PI).sin() * (1.0 - progress) * 0.7;
        transform.translation = rolling.start.lerp(rolling.target, eased) + Vec3::Y * bounce;
        let remaining_spin = (1.0 - eased) * std::f32::consts::TAU * rolling.turns;
        transform.rotation = rolling.target_rotation
            * Quat::from_euler(
                EulerRot::XYZ,
                remaining_spin,
                remaining_spin * 0.73,
                remaining_spin * 0.41,
            );
        finished &= progress >= 1.0;
    }
    if finished {
        state.notified = true;
        intents.write(PlayerIntent::RollComplete);
    }
}

fn displayed_labels(faces: &[String], selected: usize) -> Vec<&str> {
    if faces.len() == face_layouts().len() {
        return faces.iter().map(String::as_str).collect();
    }
    let mut labels = Vec::with_capacity(face_layouts().len());
    labels.push(faces[selected].as_str());
    labels.extend(
        faces
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != selected)
            .map(|(_, label)| label.as_str())
            .take(face_layouts().len() - 1),
    );
    let available = labels.len();
    while labels.len() < face_layouts().len() {
        labels.push(labels[labels.len() % available]);
    }
    labels
}

fn label_materials(
    cube_set: &CubeSet,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> HashMap<String, Handle<StandardMaterial>> {
    cube_set
        .cubes()
        .iter()
        .flat_map(|cube| cube.faces())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|label| {
            let image = images.add(label_image(&label));
            let material = materials.add(StandardMaterial {
                base_color: Color::BLACK,
                base_color_texture: Some(image),
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.8,
                ..default()
            });
            (label, material)
        })
        .collect()
}

fn label_image(label: &str) -> Image {
    let font = FontArc::try_from_slice(include_bytes!("../../../assets/fonts/Mukta-Regular.ttf"))
        .expect("Mukta-Regular.ttf must be a valid TrueType font");
    let scale = PxScale::from(if label.chars().count() == 1 {
        380.0
    } else {
        300.0
    });
    let scaled_font = font.as_scaled(scale);
    let glyph_ids: Vec<GlyphId> = label
        .chars()
        .map(|character| font.glyph_id(character))
        .collect();
    let width: f32 = glyph_ids
        .iter()
        .map(|glyph_id| scaled_font.h_advance(*glyph_id))
        .sum();
    let mut pixels = vec![0; (LABEL_IMAGE_SIZE * LABEL_IMAGE_SIZE * 4) as usize];
    let mut x = (LABEL_IMAGE_SIZE as f32 - width) / 2.0;
    let baseline = (LABEL_IMAGE_SIZE as f32 - scaled_font.height()) / 2.0 + scaled_font.ascent();

    for glyph_id in glyph_ids {
        let glyph = glyph_id.with_scale_and_position(scale, point(x, baseline));
        if let Some(outline) = font.outline_glyph(glyph) {
            let bounds = outline.px_bounds();
            outline.draw(|glyph_x, glyph_y, coverage| {
                let image_x = glyph_x as i32 + bounds.min.x as i32;
                let image_y = glyph_y as i32 + bounds.min.y as i32;
                if image_x < 0
                    || image_y < 0
                    || image_x >= LABEL_IMAGE_SIZE as i32
                    || image_y >= LABEL_IMAGE_SIZE as i32
                {
                    return;
                }
                let pixel = ((image_y as u32 * LABEL_IMAGE_SIZE + image_x as u32) * 4) as usize;
                pixels[pixel + 3] = (coverage * 255.0) as u8;
            });
        }
        x += scaled_font.h_advance(glyph_id);
    }

    Image::new(
        Extent3d {
            width: LABEL_IMAGE_SIZE,
            height: LABEL_IMAGE_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

#[derive(Clone, Copy)]
struct FaceLayout {
    normal: Vec3,
    up: Vec3,
    right: Vec3,
}

impl FaceLayout {
    fn rotation(self) -> Quat {
        Quat::from_mat3(&Mat3::from_cols(self.right, self.up, self.normal))
    }
}

fn face_layouts() -> [FaceLayout; 6] {
    [
        FaceLayout {
            normal: Vec3::Y,
            up: Vec3::NEG_Z,
            right: Vec3::X,
        },
        FaceLayout {
            normal: Vec3::NEG_Y,
            up: Vec3::Z,
            right: Vec3::X,
        },
        FaceLayout {
            normal: Vec3::X,
            up: Vec3::Y,
            right: Vec3::NEG_Z,
        },
        FaceLayout {
            normal: Vec3::NEG_X,
            up: Vec3::Y,
            right: Vec3::Z,
        },
        FaceLayout {
            normal: Vec3::Z,
            up: Vec3::Y,
            right: Vec3::X,
        },
        FaceLayout {
            normal: Vec3::NEG_Z,
            up: Vec3::Y,
            right: Vec3::NEG_X,
        },
    ]
}

pub(crate) fn grid_positions(grid_size: usize) -> Vec<Vec3> {
    let center = (grid_size.saturating_sub(1)) as f32 / 2.0;
    (0..grid_size * grid_size)
        .map(|index| {
            let row = index / grid_size;
            let column = index % grid_size;
            Vec3::new(
                (column as f32 - center) * DIE_SPACING,
                0.0,
                (row as f32 - center) * DIE_SPACING,
            )
        })
        .collect()
}

fn cube_orientations() -> [Quat; 24] {
    let resting_orientations = [
        Quat::IDENTITY,
        Quat::from_rotation_z(std::f32::consts::PI),
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
    ];
    std::array::from_fn(|index| {
        let base = resting_orientations[index / 4];
        let top_normal = base * Vec3::Y;
        Quat::from_axis_angle(top_normal, index as f32 % 4.0 * std::f32::consts::FRAC_PI_2) * base
    })
}

fn orientation_for_top_face(face_index: usize, turn: usize) -> Quat {
    let face = face_layouts()[face_index % face_layouts().len()];
    cube_orientations()
        .into_iter()
        .filter(|rotation| (rotation * face.normal).abs_diff_eq(Vec3::Y, 0.001))
        .nth(turn % 4)
        .expect("each physical cube face has four top orientations")
}

fn select_die(
    event: On<Pointer<Click>>,
    cells: Query<&DieCell>,
    mut input: ResMut<InputDraft>,
    game: Res<ActiveMatch>,
) {
    let Ok(cell) = cells.get(event.entity) else {
        return;
    };
    if game.0.phase() != MatchPhase::Playing {
        input.feedback = "Wait for the dice to settle.".into();
        return;
    }
    if input.path.contains(&cell.0) {
        input.feedback = "A die cannot be reused.".into();
        return;
    }
    let grid = game
        .0
        .current_round()
        .expect("the playing match has an active round")
        .board()
        .grid();
    if let Some(&last) = input.path.last()
        && !grid.neighbors(last).contains(&cell.0)
    {
        input.feedback = "Choose an adjacent die.".into();
        return;
    }
    input.path.push(cell.0);
    input.typed.clear();
    input.feedback = format!(
        "Selected {}",
        grid.word_for_path(&input.path)
            .expect("selected paths are validated before insertion")
            .to_uppercase()
    );
}

fn consume_highlight_requests(
    mut notices: MessageReader<MatchNotice>,
    mut requests: MessageReader<BoardHighlightRequest>,
    mut highlight: ResMut<HighlightState>,
) {
    for notice in notices.read() {
        if let MatchNotice::SubmissionAccepted(submission) = notice {
            highlight_path(&mut highlight, submission.path());
        }
    }
    for request in requests.read() {
        highlight_path(&mut highlight, &request.path);
    }
}

fn highlight_path(highlight: &mut HighlightState, path: &[usize]) {
    highlight.0 = Some(WordHighlight {
        path: path.to_vec(),
        timer: Timer::from_seconds(3.0, TimerMode::Once),
    });
}

fn tick_highlight(time: Res<Time>, mut highlight: ResMut<HighlightState>) {
    let Some(active) = highlight.0.as_mut() else {
        return;
    };
    active.timer.tick(time.delta());
    if active.timer.is_finished() {
        highlight.0 = None;
    }
}

fn draw_highlight(game: Res<ActiveMatch>, highlight: Res<HighlightState>, mut gizmos: Gizmos) {
    let Some(active) = &highlight.0 else {
        return;
    };
    let positions = grid_positions(game.0.rules().grid_size());
    for pair in active.path.windows(2) {
        let start = positions[pair[0]] + Vec3::Y * (DIE_SIZE + 0.04);
        let end = positions[pair[1]] + Vec3::Y * (DIE_SIZE + 0.04);
        gizmos.line(start, end, Color::srgb(0.9, 0.05, 0.05));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoardHighlightRequest, HighlightState, consume_highlight_requests, cube_orientations,
        displayed_labels, face_layouts, grid_positions, orientation_for_top_face,
    };
    use bevy::prelude::*;

    use crate::match_plugin::MatchNotice;

    #[test]
    fn grid_positions_are_centered_for_multiple_sizes() {
        for size in [4, 5] {
            let positions = grid_positions(size);
            assert_eq!(positions.len(), size * size);
            assert!(positions.iter().all(|position| position.y == 0.0));
            assert!((positions.iter().map(|position| position.x).sum::<f32>()).abs() < 0.001);
            assert!((positions.iter().map(|position| position.z).sum::<f32>()).abs() < 0.001);
            for (index, position) in positions.iter().enumerate() {
                assert!(!positions[..index].contains(position));
            }
        }
    }

    #[test]
    fn every_face_can_be_authoritatively_oriented_up() {
        for (face_index, face) in face_layouts().into_iter().enumerate() {
            for turn in 0..4 {
                let orientation = orientation_for_top_face(face_index, turn);
                assert!((orientation * face.normal).abs_diff_eq(Vec3::Y, 0.001));
            }
        }
        let orientations = cube_orientations();
        for (index, orientation) in orientations.iter().enumerate() {
            assert!(
                orientations[..index]
                    .iter()
                    .all(|other| !orientation.abs_diff_eq(*other, 0.001)
                        && !orientation.abs_diff_eq(-*other, 0.001))
            );
        }
    }

    #[test]
    fn non_physical_cube_sets_still_display_the_selected_label_on_top() {
        let faces = ["a", "b", "c", "d", "e", "f", "g"]
            .map(String::from)
            .to_vec();
        assert_eq!(displayed_labels(&faces, 6)[0], "g");

        let one_face = vec![String::from("qu")];
        assert_eq!(
            displayed_labels(&one_face, 0),
            vec!["qu", "qu", "qu", "qu", "qu", "qu"]
        );
    }

    #[test]
    fn board_highlight_requests_use_the_existing_word_path_visualization() {
        let mut app = App::new();
        app.add_message::<MatchNotice>()
            .add_message::<BoardHighlightRequest>()
            .init_resource::<HighlightState>()
            .add_systems(Update, consume_highlight_requests);

        app.world_mut()
            .write_message(BoardHighlightRequest {
                path: vec![0, 1, 5],
            })
            .unwrap();
        app.update();

        assert_eq!(
            app.world()
                .resource::<HighlightState>()
                .0
                .as_ref()
                .map(|highlight| highlight.path.as_slice()),
            Some([0, 1, 5].as_slice())
        );
    }
}
