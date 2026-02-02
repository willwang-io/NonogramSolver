use crate::one_line_solver::OneLineSolver;
use crate::puzzle_crawler::{Group, PuzzleData};

#[derive(Debug, Clone, PartialEq)]
pub struct SolvedPuzzle {
    pub color_panel: Vec<String>,
    pub grid: Vec<Vec<u64>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolveSteps {
    pub color_panel: Vec<String>,
    pub steps: Vec<Vec<Vec<u64>>>,
}

#[derive(Debug)]
pub enum SolveError {
    TooManyColors(usize),
    Unsolvable,
}

impl std::fmt::Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyColors(count) => {
                write!(f, "too many colors to fit in a u64 mask: {count}")
            }
            Self::Unsolvable => write!(f, "puzzle cannot be solved with current constraints"),
        }
    }
}

impl std::error::Error for SolveError {}

pub fn solve_puzzle(data: PuzzleData) -> Result<SolvedPuzzle, SolveError> {
    let steps = solve_puzzle_steps(data)?;
    let grid = steps
        .steps
        .last()
        .cloned()
        .ok_or(SolveError::Unsolvable)?;
    Ok(SolvedPuzzle {
        color_panel: steps.color_panel,
        grid,
    })
}

pub fn solve_puzzle_steps(data: PuzzleData) -> Result<SolveSteps, SolveError> {
    let color_count = data.color_panel.len();
    if color_count == 0 || color_count > 63 {
        return Err(SolveError::TooManyColors(color_count));
    }
    let full_mask = (1u64 << color_count) - 1;

    let row_groups = convert_groups(&data.row_groups);
    let col_groups = convert_groups(&data.col_groups);

    let m = row_groups.len();
    let n = col_groups.len();

    let mut row_masks = vec![vec![full_mask; n]; m];
    let mut col_masks = vec![vec![full_mask; m]; n];

    let mut dead_rows = vec![false; m];
    let mut dead_cols = vec![false; n];
    let mut solver = OneLineSolver::new(m.max(n));

    let mut steps = Vec::new();
    steps.push(row_masks.clone());

    let mut prev_sum = u64::MAX;
    loop {
        if !update_groups_state(&mut solver, &mut dead_rows, &row_groups, &mut row_masks) {
            return Err(SolveError::Unsolvable);
        }
        if !update_groups_state(&mut solver, &mut dead_cols, &col_groups, &mut col_masks) {
            return Err(SolveError::Unsolvable);
        }

        let cur_sum = update_cell_values(&mut row_masks, &mut col_masks);
        if cur_sum == prev_sum {
            break;
        }
        prev_sum = cur_sum;
        steps.push(row_masks.clone());
    }

    Ok(SolveSteps {
        color_panel: data.color_panel,
        steps,
    })
}

fn convert_groups(groups: &[Vec<Group>]) -> Vec<Vec<(usize, usize)>> {
    groups
        .iter()
        .map(|row| row.iter().map(|g| (g.len, g.color_id)).collect())
        .collect()
}

fn update_groups_state(
    solver: &mut OneLineSolver,
    dead: &mut [bool],
    groups: &[Vec<(usize, usize)>],
    masks: &mut [Vec<u64>],
) -> bool {
    for (idx, group) in groups.iter().enumerate() {
        if dead[idx] {
            continue;
        }
        if !solver.update_state(group, &mut masks[idx]) {
            return false;
        }
        dead[idx] = masks[idx].iter().all(|mask| is_single_bit(*mask));
    }
    true
}

fn update_cell_values(row_masks: &mut [Vec<u64>], col_masks: &mut [Vec<u64>]) -> u64 {
    let mut total: u64 = 0;
    for row in 0..row_masks.len() {
        for col in 0..row_masks[row].len() {
            let combined = row_masks[row][col] & col_masks[col][row];
            row_masks[row][col] = combined;
            col_masks[col][row] = combined;
            total = total.wrapping_add(combined);
        }
    }
    total
}

fn is_single_bit(mask: u64) -> bool {
    mask != 0 && (mask & (mask - 1)) == 0
}

pub fn mask_to_color_index(mask: u64) -> Option<usize> {
    if is_single_bit(mask) {
        Some(mask.trailing_zeros() as usize)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle_crawler::Group;

    fn masks_from_color_ids(grid: &[Vec<usize>]) -> Vec<Vec<u64>> {
        grid.iter()
            .map(|row| row.iter().map(|id| 1u64 << id).collect())
            .collect()
    }

    fn groups_from_grid(grid: &[Vec<u64>]) -> (Vec<Vec<Group>>, Vec<Vec<Group>>) {
        let rows = grid.len();
        let cols = grid.first().map(|row| row.len()).unwrap_or(0);
        let mut row_groups = Vec::with_capacity(rows);
        for row in grid {
            let mut groups = Vec::new();
            let mut col = 0;
            while col < cols {
                let color = mask_to_color_index(row[col]).expect("unsolved cell");
                let start = col;
                while col < cols && mask_to_color_index(row[col]).unwrap() == color {
                    col += 1;
                }
                let len = col - start;
                if color > 0 && len > 0 {
                    groups.push(Group { len, color_id: color });
                }
            }
            row_groups.push(groups);
        }

        let mut col_groups = Vec::with_capacity(cols);
        for col in 0..cols {
            let mut groups = Vec::new();
            let mut row = 0;
            while row < rows {
                let color = mask_to_color_index(grid[row][col]).expect("unsolved cell");
                let start = row;
                while row < rows && mask_to_color_index(grid[row][col]).unwrap() == color {
                    row += 1;
                }
                let len = row - start;
                if color > 0 && len > 0 {
                    groups.push(Group { len, color_id: color });
                }
            }
            col_groups.push(groups);
        }
        (row_groups, col_groups)
    }

    #[test]
    fn solves_black_white_puzzle_matches_hints() {
        let solved_ids = vec![
            vec![0, 0, 1, 0, 0],
            vec![0, 1, 1, 1, 0],
            vec![1, 1, 1, 1, 1],
            vec![0, 1, 1, 1, 0],
            vec![0, 0, 1, 0, 0],
        ];
        let solved_masks = masks_from_color_ids(&solved_ids);
        let (row_groups, col_groups) = groups_from_grid(&solved_masks);

        let puzzle = PuzzleData {
            color_panel: vec!["#ffffff".to_string(), "#000000".to_string()],
            row_groups,
            col_groups,
        };

        let solved = solve_puzzle(puzzle.clone()).expect("puzzle should solve");
        let (row_out, col_out) = groups_from_grid(&solved.grid);
        assert_eq!(row_out, puzzle.row_groups);
        assert_eq!(col_out, puzzle.col_groups);
    }

    #[test]
    fn solves_color_puzzle_matches_hints() {
        let solved_ids = vec![
            vec![1, 1, 1],
            vec![2, 2, 2],
            vec![1, 1, 1],
        ];
        let solved_masks = masks_from_color_ids(&solved_ids);
        let (row_groups, col_groups) = groups_from_grid(&solved_masks);

        let puzzle = PuzzleData {
            color_panel: vec![
                "#ffffff".to_string(),
                "#ff0000".to_string(),
                "#0000ff".to_string(),
            ],
            row_groups,
            col_groups,
        };

        let solved = solve_puzzle(puzzle.clone()).expect("puzzle should solve");
        let (row_out, col_out) = groups_from_grid(&solved.grid);
        assert_eq!(row_out, puzzle.row_groups);
        assert_eq!(col_out, puzzle.col_groups);
    }
}
