// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use bevy::asset::RenderAssetUsages;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::prelude::*;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::prelude::*;
use std::collections::HashMap;
use word_grid_game::{CubeSet, GameDefinition, STANDARD_NEW_DICE, SinglePlayerSession};
use word_grid_solver::{Dictionary, WordGrid};

use ab_glyph::{Font, FontArc, GlyphId, PxScale, ScaleFont, point};

const GRID_SIDE: usize = 4;
const DIE_SIZE: f32 = 1.0;
const DIE_SPACING: f32 = 1.25;
const FACE_OFFSET: f32 = DIE_SIZE / 2.0 + 0.003;
const LABEL_IMAGE_SIZE: u32 = 512;
const LABEL_FACE_SIZE: f32 = 1.64;

#[derive(Component)]
struct DieCell(usize);

#[derive(Component)]
struct HudInput;

#[derive(Component)]
struct HudFeedback;

#[derive(Component)]
struct RoundTotal;

#[derive(Component)]
struct WordList;

#[derive(Resource)]
struct GameState(SinglePlayerSession);

#[derive(Resource, Default)]
struct InputState {
    path: Vec<usize>,
    typed: String,
    feedback: String,
}

#[derive(Resource, Default)]
struct HighlightState(Option<WordHighlight>);

struct WordHighlight {
    path: Vec<usize>,
    timer: Timer,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Word Grid".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                keyboard_input,
                scroll_found_words,
                tick_highlight,
                draw_highlight,
                refresh_hud,
            ),
        )
        .run();
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(1.0, 0.93, 0.82),
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });

    let label_mesh = meshes.add(Rectangle::new(LABEL_FACE_SIZE, LABEL_FACE_SIZE));
    let table_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.31, 0.14, 0.06),
        perceptual_roughness: 0.78,
        ..default()
    });
    let label_materials = label_materials(&mut images, &mut materials);

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(table_material),
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.9, 0.75),
            illuminance: 4_500.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-3.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.0, 8.5, 0.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
    ));

    let orientations = cube_orientations();
    let mut rng = rand::rng();
    let mut cells = Vec::with_capacity(GRID_SIDE * GRID_SIDE);
    for (index, labels) in STANDARD_NEW_DICE.iter().enumerate() {
        let position = grid_positions()[index];
        let rotation = *orientations
            .choose(&mut rng)
            .expect("cube orientations are not empty");
        let top_label = face_layouts()
            .into_iter()
            .zip(labels)
            .find_map(|(face, label)| {
                ((rotation * face.normal).abs_diff_eq(Vec3::Y, 0.001)).then_some(*label)
            })
            .expect("every cube orientation has a top face");
        cells.push(top_label.to_string());
        let mut die = commands.spawn((
            Name::new(format!("Die {}", index + 1)),
            DieCell(index),
            Pickable::default(),
            WorldAssetRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/game_cube.glb")),
            ),
            Transform::from_translation(position + Vec3::Y * (DIE_SIZE / 2.0))
                .with_rotation(rotation),
        ));

        die.observe(select_die);
        die.with_children(|parent| {
            for (face, label) in face_layouts().into_iter().zip(labels) {
                parent.spawn((
                    Mesh3d(label_mesh.clone()),
                    MeshMaterial3d(label_materials[label].clone()),
                    Transform::from_translation(face.normal * FACE_OFFSET)
                        .with_rotation(face.rotation()),
                ));
            }
        });
    }

    let dictionary = Dictionary::from_file("twl06.txt").expect("twl06.txt must be available");
    let mut session = SinglePlayerSession::new(
        GameDefinition::normal(CubeSet::StandardNew),
        dictionary,
        &mut rng,
    );
    session.board.grid = WordGrid::new(GRID_SIDE, cells).expect("the rendered board is square");
    commands.insert_resource(GameState(session));
    commands.insert_resource(InputState {
        feedback: "Click adjacent dice; type a word; Enter submits.".into(),
        ..default()
    });
    commands.insert_resource(HighlightState::default());
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(300.0),
                height: Val::Px(560.0),
                padding: UiRect::all(Val::Px(16.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.06, 0.04, 0.88)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("WORD GRID"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor::WHITE,
            ));
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(42.0),
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                ))
                .with_children(|input| {
                    input.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor::BLACK,
                        HudInput,
                    ));
                });
            parent.spawn((
                Text::new("Click dice or type a word, then press Enter."),
                TextFont {
                    font_size: FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.7, 0.5)),
                HudFeedback,
            ));
            parent.spawn((
                Text::new("FOUND WORDS"),
                TextFont {
                    font_size: FontSize::Px(17.0),
                    ..default()
                },
                TextColor::WHITE,
            ));
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                WordList,
            ));
            parent.spawn((
                Text::new("ROUND TOTAL: 0"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor::WHITE,
                RoundTotal,
            ));
        });
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

fn label_materials(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> HashMap<&'static str, Handle<StandardMaterial>> {
    STANDARD_NEW_DICE
        .iter()
        .flatten()
        .copied()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .map(|label| {
            let image = images.add(label_image(label));
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
    let font = FontArc::try_from_slice(include_bytes!("../assets/fonts/Mukta-Regular.ttf"))
        .expect("Mukta-Regular.ttf must be a valid TrueType font");
    let scale = PxScale::from(if label.len() == 1 { 380.0 } else { 300.0 });
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

fn grid_positions() -> [Vec3; 16] {
    std::array::from_fn(|index| {
        let row = index / GRID_SIDE;
        let column = index % GRID_SIDE;
        Vec3::new(
            (column as f32 - 1.5) * DIE_SPACING,
            0.0,
            (row as f32 - 1.5) * DIE_SPACING,
        )
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_new_set_has_six_faces_on_each_of_sixteen_dice() {
        assert_eq!(STANDARD_NEW_DICE.len(), 16);
        assert!(STANDARD_NEW_DICE.iter().all(|die| die.len() == 6));
        assert!(
            STANDARD_NEW_DICE
                .iter()
                .flatten()
                .any(|label| *label == "Qu")
        );
    }

    #[test]
    fn grid_positions_are_centered_and_distinct() {
        let positions = grid_positions();
        assert_eq!(positions.len(), 16);
        assert!(positions.iter().all(|position| position.y == 0.0));
        assert!((positions.iter().map(|position| position.x).sum::<f32>()).abs() < f32::EPSILON);
        assert!((positions.iter().map(|position| position.z).sum::<f32>()).abs() < f32::EPSILON);
        for (index, position) in positions.iter().enumerate() {
            assert!(!positions[..index].contains(position));
        }
    }

    #[test]
    fn cube_orientations_are_unique_right_angle_rotations() {
        let orientations = cube_orientations();
        assert_eq!(orientations.len(), 24);
        for orientation in orientations {
            for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
                let rotated = orientation * axis;
                assert!(rotated.x.abs() < 0.001 || (rotated.x.abs() - 1.0).abs() < 0.001);
                assert!(rotated.y.abs() < 0.001 || (rotated.y.abs() - 1.0).abs() < 0.001);
                assert!(rotated.z.abs() < 0.001 || (rotated.z.abs() - 1.0).abs() < 0.001);
            }
        }
        for (index, orientation) in orientations.iter().enumerate() {
            assert!(
                orientations[..index]
                    .iter()
                    .all(|other| orientation.abs_diff_eq(*other, 0.001) == false
                        && orientation.abs_diff_eq(-*other, 0.001) == false)
            );
        }
    }

    #[test]
    fn backspace_removes_typed_text_before_a_selected_path() {
        let mut input = InputState {
            path: vec![1, 2],
            typed: "cat".into(),
            ..default()
        };

        delete_last_input(&mut input);
        assert_eq!(input.typed, "ca");
        assert_eq!(input.path, vec![1, 2]);

        input.typed.clear();
        delete_last_input(&mut input);
        assert_eq!(input.path, vec![1]);
    }
}

fn select_die(
    event: On<Pointer<Click>>,
    cells: Query<&DieCell>,
    mut input: ResMut<InputState>,
    state: Res<GameState>,
) {
    let Ok(cell) = cells.get(event.entity) else {
        return;
    };
    if input.path.contains(&cell.0) {
        input.feedback = "A die cannot be reused.".into();
        return;
    }
    if let Some(&last) = input.path.last()
        && !state.0.board.grid.neighbors(last).contains(&cell.0)
    {
        input.feedback = "Choose an adjacent die.".into();
        return;
    }
    input.path.push(cell.0);
    input.typed.clear();
    input.feedback = format!(
        "Selected {}",
        state
            .0
            .board
            .grid
            .word_for_path(&input.path)
            .expect("selected paths are validated before insertion")
            .to_uppercase()
    );
}

fn keyboard_input(
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut input: ResMut<InputState>,
    mut state: ResMut<GameState>,
    mut highlight: ResMut<HighlightState>,
) {
    for event in events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        if event.key_code == KeyCode::Backspace {
            delete_last_input(&mut input);
        } else if event.key_code == KeyCode::Enter {
            // Enter may carry a newline in `text`; it submits and is never part of a word.
            continue;
        } else if let Some(text) = &event.text {
            input.typed.push_str(text);
            input.path.clear();
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        input.path.clear();
        input.typed.clear();
    }
    if keys.just_pressed(KeyCode::Enter) {
        let result = if input.typed.is_empty() {
            state.0.submit_path(input.path.clone())
        } else {
            state.0.submit_text(&input.typed)
        };
        input.feedback = match result {
            Ok(word) => {
                highlight.0 = Some(WordHighlight {
                    path: word.path.clone(),
                    timer: Timer::from_seconds(3.0, TimerMode::Once),
                });
                format!("Accepted {} (+{})", word.word.to_uppercase(), word.score)
            }
            Err(error) => error.to_string(),
        };
        input.path.clear();
        input.typed.clear();
    }
}

fn delete_last_input(input: &mut InputState) {
    if input.typed.is_empty() {
        input.path.pop();
    } else {
        input.typed.pop();
    }
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

fn draw_highlight(highlight: Res<HighlightState>, mut gizmos: Gizmos) {
    let Some(active) = &highlight.0 else {
        return;
    };

    for pair in active.path.windows(2) {
        let start = grid_positions()[pair[0]] + Vec3::Y * (DIE_SIZE + 0.04);
        let end = grid_positions()[pair[1]] + Vec3::Y * (DIE_SIZE + 0.04);
        gizmos.line(start, end, Color::srgb(0.9, 0.05, 0.05));
    }
}

fn scroll_found_words(
    mut wheel_events: MessageReader<MouseWheel>,
    mut lists: Query<(&mut ScrollPosition, &ComputedNode), With<WordList>>,
) {
    let mut delta = 0.0;
    for event in wheel_events.read() {
        delta -= event.y
            * if event.unit == MouseScrollUnit::Line {
                24.0
            } else {
                1.0
            };
    }
    if delta == 0.0 {
        return;
    }

    for (mut position, computed) in &mut lists {
        let max_offset = (computed.content_size().y - computed.size().y).max(0.0)
            * computed.inverse_scale_factor();
        position.y = (position.y + delta).clamp(0.0, max_offset);
    }
}

fn refresh_hud(
    state: Res<GameState>,
    input: Res<InputState>,
    mut commands: Commands,
    mut texts: ParamSet<(
        Query<&mut Text, With<HudInput>>,
        Query<&mut Text, With<HudFeedback>>,
        Query<&mut Text, With<RoundTotal>>,
    )>,
    word_list: Query<Entity, With<WordList>>,
) {
    let word = if input.typed.is_empty() {
        state
            .0
            .board
            .grid
            .word_for_path(&input.path)
            .unwrap_or_default()
    } else {
        input.typed.clone()
    };
    if let Ok(mut text) = texts.p0().single_mut() {
        text.0 = word.to_uppercase();
    }
    if let Ok(mut text) = texts.p1().single_mut() {
        text.0 = input.feedback.clone();
    }
    if let Ok(mut text) = texts.p2().single_mut() {
        text.0 = format!("ROUND TOTAL: {}", state.0.round_score());
    }

    if !state.is_changed() {
        return;
    }
    let Ok(list) = word_list.single() else {
        return;
    };
    commands.entity(list).despawn_related::<Children>();
    let mut words = state.0.round_words.clone();
    words.sort_by(|left, right| {
        right
            .word
            .chars()
            .count()
            .cmp(&left.word.chars().count())
            .then_with(|| left.word.cmp(&right.word))
    });
    commands.entity(list).with_children(|parent| {
        for found in words {
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(found.word.to_uppercase()),
                        TextFont {
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor::WHITE,
                    ));
                    row.spawn((
                        Text::new(found.score.to_string()),
                        TextFont {
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.82, 0.35)),
                    ));
                });
        }
    });
}
