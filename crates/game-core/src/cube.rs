// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cube {
    faces: Vec<String>,
}

impl Cube {
    pub fn new(faces: impl IntoIterator<Item = impl Into<String>>) -> Result<Self, CubeSetError> {
        let faces: Vec<_> = faces
            .into_iter()
            .map(Into::into)
            .map(|face: String| face.trim().to_string())
            .collect();
        if faces.is_empty() {
            return Err(CubeSetError::CubeHasNoFaces);
        }
        if let Some(index) = faces.iter().position(String::is_empty) {
            return Err(CubeSetError::EmptyFace { index });
        }
        Ok(Self { faces })
    }

    pub fn faces(&self) -> &[String] {
        &self.faces
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CubeSet {
    name: String,
    cubes: Vec<Cube>,
}

impl CubeSet {
    pub fn new(
        name: impl Into<String>,
        cubes: impl IntoIterator<Item = Cube>,
    ) -> Result<Self, CubeSetError> {
        let name = name.into().trim().to_string();
        if name.is_empty() {
            return Err(CubeSetError::EmptyName);
        }
        let cubes: Vec<_> = cubes.into_iter().collect();
        if cubes.is_empty() {
            return Err(CubeSetError::NoCubes);
        }
        Ok(Self { name, cubes })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn cubes(&self) -> &[Cube] {
        &self.cubes
    }

    pub fn standard_new() -> Self {
        cube_set_from_faces("Standard New", &STANDARD_NEW_FACES)
    }

    pub fn standard_old() -> Self {
        cube_set_from_faces("Standard Old", &STANDARD_OLD_FACES)
    }

    pub fn big() -> Self {
        cube_set_from_faces("Big", &BIG_FACES)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CubeSetError {
    EmptyName,
    NoCubes,
    CubeHasNoFaces,
    EmptyFace { index: usize },
}

impl std::fmt::Display for CubeSetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "cube set name cannot be empty"),
            Self::NoCubes => write!(formatter, "cube set must contain at least one cube"),
            Self::CubeHasNoFaces => write!(formatter, "cube must contain at least one face"),
            Self::EmptyFace { index } => write!(formatter, "cube face {index} cannot be empty"),
        }
    }
}

impl std::error::Error for CubeSetError {}

fn cube_set_from_faces<const N: usize>(name: &str, faces: &[[&str; 6]; N]) -> CubeSet {
    let cubes = faces
        .iter()
        .map(|faces| Cube::new(faces.iter().copied()).expect("built-in cube faces are valid"));
    CubeSet::new(name, cubes).expect("built-in cube set is valid")
}

const STANDARD_NEW_FACES: [[&str; 6]; 16] = [
    ["A", "E", "A", "N", "E", "G"],
    ["A", "H", "S", "P", "C", "O"],
    ["A", "S", "P", "F", "F", "K"],
    ["O", "B", "J", "O", "A", "B"],
    ["I", "O", "T", "M", "U", "C"],
    ["R", "Y", "V", "D", "E", "L"],
    ["L", "R", "E", "I", "X", "D"],
    ["E", "I", "U", "N", "E", "S"],
    ["W", "N", "G", "E", "E", "H"],
    ["L", "N", "H", "N", "R", "Z"],
    ["T", "S", "T", "I", "Y", "D"],
    ["O", "W", "T", "O", "A", "T"],
    ["E", "R", "T", "T", "Y", "L"],
    ["T", "O", "E", "S", "S", "I"],
    ["T", "E", "R", "W", "H", "V"],
    ["N", "U", "I", "H", "M", "Qu"],
];

const STANDARD_OLD_FACES: [[&str; 6]; 16] = [
    ["A", "A", "C", "I", "O", "T"],
    ["A", "B", "I", "L", "T", "Y"],
    ["A", "B", "J", "M", "O", "Qu"],
    ["A", "C", "D", "E", "M", "P"],
    ["A", "C", "E", "L", "R", "S"],
    ["A", "D", "E", "N", "V", "Z"],
    ["A", "H", "M", "O", "R", "S"],
    ["B", "I", "F", "O", "R", "X"],
    ["D", "E", "N", "O", "S", "W"],
    ["D", "K", "N", "O", "T", "U"],
    ["E", "E", "F", "H", "I", "Y"],
    ["E", "G", "K", "L", "U", "Y"],
    ["E", "G", "I", "N", "T", "V"],
    ["E", "H", "I", "N", "P", "S"],
    ["E", "L", "P", "S", "T", "U"],
    ["G", "I", "L", "R", "U", "W"],
];

const BIG_FACES: [[&str; 6]; 25] = [
    ["A", "A", "A", "F", "R", "S"],
    ["A", "A", "E", "E", "E", "E"],
    ["A", "A", "F", "I", "R", "S"],
    ["A", "D", "E", "N", "N", "N"],
    ["A", "E", "E", "E", "E", "M"],
    ["A", "E", "E", "G", "M", "U"],
    ["A", "E", "G", "M", "N", "N"],
    ["A", "F", "I", "R", "S", "Y"],
    ["B", "J", "K", "Q", "X", "Z"],
    ["C", "C", "E", "N", "S", "T"],
    ["C", "E", "I", "I", "L", "T"],
    ["C", "E", "I", "L", "P", "T"],
    ["C", "E", "I", "P", "S", "T"],
    ["D", "D", "H", "N", "O", "T"],
    ["D", "H", "H", "L", "O", "R"],
    ["D", "H", "L", "N", "O", "R"],
    ["D", "H", "L", "N", "O", "R"],
    ["E", "I", "I", "I", "T", "T"],
    ["E", "M", "O", "T", "T", "T"],
    ["E", "N", "S", "S", "S", "U"],
    ["F", "I", "P", "R", "S", "Y"],
    ["G", "O", "R", "R", "V", "W"],
    ["I", "P", "R", "R", "R", "Y"],
    ["N", "O", "O", "T", "U", "W"],
    ["O", "O", "O", "T", "T", "U"],
];

#[cfg(test)]
mod tests {
    use super::{Cube, CubeSet, CubeSetError};

    #[test]
    fn validates_custom_cube_data() {
        assert_eq!(
            Cube::new(Vec::<String>::new()),
            Err(CubeSetError::CubeHasNoFaces)
        );
        assert_eq!(
            Cube::new(["a", " "]),
            Err(CubeSetError::EmptyFace { index: 1 })
        );
        assert_eq!(
            CubeSet::new(" ", [Cube::new(["a"]).unwrap()]),
            Err(CubeSetError::EmptyName)
        );
    }

    #[test]
    fn built_in_sets_have_expected_sizes() {
        assert_eq!(CubeSet::standard_new().cubes().len(), 16);
        assert_eq!(CubeSet::standard_old().cubes().len(), 16);
        assert_eq!(CubeSet::big().cubes().len(), 25);
        assert!(
            CubeSet::standard_new()
                .cubes()
                .iter()
                .flat_map(Cube::faces)
                .any(|face| face == "Qu")
        );
    }
}
