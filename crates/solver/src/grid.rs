#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordGrid {
    size: usize,
    cells: Vec<String>,
}

impl WordGrid {
    pub fn new(size: usize, cells: Vec<String>) -> Result<Self, String> {
        if size == 0 {
            return Err("grid size must be greater than 0".to_string());
        }

        if cells.len() != size * size {
            return Err(format!(
                "grid has {} cells, but size {} requires {} cells",
                cells.len(),
                size,
                size * size
            ));
        }

        let mut normalized = Vec::with_capacity(cells.len());
        for cell in cells {
            let cell = cell.trim().to_ascii_lowercase();
            if cell.is_empty() {
                return Err("grid cells cannot be empty".to_string());
            }
            normalized.push(cell);
        }

        Ok(Self {
            size,
            cells: normalized,
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn cells(&self) -> &[String] {
        &self.cells
    }

    pub fn cell(&self, index: usize) -> &str {
        &self.cells[index]
    }

    pub fn neighbors(&self, index: usize) -> Vec<usize> {
        let row = index / self.size;
        let col = index % self.size;
        let row_start = row.saturating_sub(1);
        let row_end = (row + 1).min(self.size - 1);
        let col_start = col.saturating_sub(1);
        let col_end = (col + 1).min(self.size - 1);
        let mut neighbors = Vec::new();

        for next_row in row_start..=row_end {
            for next_col in col_start..=col_end {
                let next = next_row * self.size + next_col;
                if next != index {
                    neighbors.push(next);
                }
            }
        }

        neighbors
    }
}

#[cfg(test)]
mod tests {
    use super::WordGrid;

    #[test]
    fn validates_square_grid_cell_count() {
        let err = WordGrid::new(2, vec!["a".into(), "b".into(), "c".into()])
            .expect_err("grid should reject too few cells");

        assert!(err.contains("requires 4 cells"));
    }

    #[test]
    fn rejects_empty_cells() {
        let err = WordGrid::new(1, vec![" ".into()]).expect_err("empty cell should fail");

        assert_eq!(err, "grid cells cannot be empty");
    }

    #[test]
    fn finds_neighbors_for_corner_and_center() {
        let grid = WordGrid::new(
            3,
            ["a", "b", "c", "d", "e", "f", "g", "h", "i"]
                .into_iter()
                .map(String::from)
                .collect(),
        )
        .expect("valid grid");

        assert_eq!(grid.neighbors(0), vec![1, 3, 4]);
        assert_eq!(grid.neighbors(4), vec![0, 1, 2, 3, 5, 6, 7, 8]);
    }
}
