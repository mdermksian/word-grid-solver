// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use rand::{Rng, RngExt};
use word_grid_solver::WordGrid;

use crate::GameRules;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RolledCube {
    cube_index: usize,
    face_index: usize,
    label: String,
}

impl RolledCube {
    pub fn cube_index(&self) -> usize {
        self.cube_index
    }

    pub fn face_index(&self) -> usize {
        self.face_index
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardRoll {
    grid: WordGrid,
    cubes: Vec<RolledCube>,
}

impl BoardRoll {
    pub fn roll(rules: &GameRules, rng: &mut impl Rng) -> Self {
        let cube_set = rules.cube_set();
        let mut cube_indices: Vec<_> = (0..cube_set.cubes().len()).collect();
        for index in (1..cube_indices.len()).rev() {
            cube_indices.swap(index, rng.random_range(0..=index));
        }

        let cubes: Vec<_> = cube_indices
            .into_iter()
            .map(|cube_index| {
                let cube = &cube_set.cubes()[cube_index];
                let face_index = rng.random_range(0..cube.faces().len());
                RolledCube {
                    cube_index,
                    face_index,
                    label: cube.faces()[face_index].clone(),
                }
            })
            .collect();
        let cells = cubes.iter().map(|cube| cube.label.clone()).collect();
        let grid = WordGrid::new(rules.grid_size(), cells)
            .expect("validated rules always produce a square board");
        Self { grid, cubes }
    }

    pub fn grid(&self) -> &WordGrid {
        &self.grid
    }

    pub fn cubes(&self) -> &[RolledCube] {
        &self.cubes
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use crate::{BoardRoll, CubeSet, GameRules};

    #[test]
    fn seeded_roll_is_deterministic_and_keeps_cube_identity() {
        let rules = GameRules::normal(CubeSet::standard_new()).unwrap();
        let mut first_rng = rand::rngs::SmallRng::seed_from_u64(12);
        let mut second_rng = rand::rngs::SmallRng::seed_from_u64(12);
        let first = BoardRoll::roll(&rules, &mut first_rng);
        let second = BoardRoll::roll(&rules, &mut second_rng);

        assert_eq!(first, second);
        assert_eq!(first.grid().cells().len(), 16);
        let mut indices: Vec<_> = first.cubes().iter().map(|cube| cube.cube_index()).collect();
        indices.sort_unstable();
        assert_eq!(indices, (0..16).collect::<Vec<_>>());
    }
}
